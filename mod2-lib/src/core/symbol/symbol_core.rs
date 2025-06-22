use std::{
  fmt::Display,
  sync::atomic::{AtomicU32, Ordering},
  ops::DerefMut
};
use mod2_abs::{debug, int_to_subscript, IString};

use crate::{
  api::{
    Arity,
    DagNodePtr,
    MaybeExtensionInfo,
    MaybeSubproblem
  },
  core::{
    symbol::{
      SortTable,
      SymbolAttributes,
      SymbolType,
      OpDeclaration
    },
    format::{
      FormatStyle,
      Formattable
    },
    pre_equation::{
      SortConstraintTable,
      PreEquationKind,
      PreEquationPtr as EquationPtr,
      PreEquationPtr
    },
    gc::ok_to_collect_garbage,
    rewriting_context::RewritingContext,
    sort::SortIndex,
    strategy::Strategy,
  },
  HashType,
};


pub struct SymbolCore {
  pub name       : IString,
  pub attributes : SymbolAttributes,
  pub symbol_type: SymbolType,

  pub sort_table: SortTable,

  /// The sort constraint table encapsulates functionality related to all membership constraints associated to
  /// this symbol. The "equation table" is much simpler, so we fold it into `SymbolCore`. In both cases, the
  /// `accept_*` and compiler methods are virtual, so they live in the `Symbol` trait.
  pub sort_constraint_table: SortConstraintTable,
  pub equations: Vec<EquationPtr>,

  /// > 0 for symbols that only produce an unique sort; -1 for fast case; 0 for slow case
  /// implemented as `SortIndex::FAST_CASE_UNIQUE_SORT`, `SortIndex::SLOW_CASE_UNIQUE_SORT`, and a normal value.
  pub unique_sort_index: SortIndex,

  /// Unique integer for comparing symbols, also called order. In Maude, the `order`
  /// has lower bits equal to the value of an integer that is incremented every time
  /// a symbol is created and upper 8 bits (bits 24..32) equal to the arity. Note:
  /// We enforce symbol creation with `Symbol::new()` by making hash_value private.
  hash_value : HashType,

  // ToDo: Possibly replace with `Option<Box<Strategy>>`, where `None` means "standard strategy".
  // `Strategy`
  pub(crate) strategy: Option<Box<Strategy>>,
}

// This is an abomination. See `api/built_in/mod.rs`.
unsafe impl Send for SymbolCore {}
unsafe impl Sync for SymbolCore {}

impl SymbolCore {
  /// All symbols must be created with `Symbol::new()`. If attributes, arity, symbol_type unknown, use defaults.
  pub fn new(
      name       : IString,
      arity      : Arity,
      attributes : SymbolAttributes,
      symbol_type: SymbolType,
    ) -> SymbolCore
  {
    // Compute hash
    static SYMBOL_COUNT: AtomicU32 = AtomicU32::new(0);
    SYMBOL_COUNT.fetch_add(1, Ordering::Relaxed);
    let numeric_arity: HashType = arity.get() as HashType;
    let hash_value = SYMBOL_COUNT.load(Ordering::Relaxed) | (numeric_arity << 24); // Maude: self.arity << 24

    let symbol = SymbolCore {
      name,
      attributes,
      symbol_type,
      sort_table           : SortTable::with_arity(arity),
      sort_constraint_table: SortConstraintTable::new(),
      equations            : vec![],
      unique_sort_index    : SortIndex::UNKNOWN,
      hash_value,
      strategy             : Default::default(),
    };

    symbol
  }

  #[inline(always)]
  pub fn with_arity(name: IString, arity: Arity)  -> SymbolCore {
    SymbolCore::new(name, arity, SymbolAttributes::default(), SymbolType::default())
  }

  #[inline(always)]
  pub fn with_name(name: IString)  -> SymbolCore {
    SymbolCore::new(name, Arity::ZERO, SymbolAttributes::default(), SymbolType::default())
  }

  #[inline(always)]
  pub fn is_variable(&self) -> bool {
    self.symbol_type == SymbolType::Variable
  }

  #[inline(always)]
  pub fn compare(&self, other: &SymbolCore) -> std::cmp::Ordering {
    self.hash_value.cmp(&other.hash_value)
  }

  /// The hash value of symbols is created on symbol creation in `Symbol::new()`
  #[inline(always)]
  pub fn hash(&self) -> HashType {
    self.hash_value
  }

  pub fn add_op_declaration(&mut self, op_declaration: OpDeclaration) {
    self.sort_table.add_op_declaration(op_declaration);
  }

  #[inline(always)]
  pub(crate) fn arity(&self) -> Arity {
    self.sort_table.arity()
  }

  // region EquationTable methods

  pub(crate) fn apply_replace(&mut self, subject: DagNodePtr, context: &mut RewritingContext, extension_info: MaybeExtensionInfo) -> bool {
    // We have to use this brain dead pattern, because the `for x in y` syntax holds an immutable borrow
    // of `self`.
    for eq_idx in 0..self.equations.len() {
      let mut eq = self.equations[eq_idx];
      // ToDo: Get rid of this atrocity:
      let mut eq2 = eq;
      let eq3 = eq;
      
      // Destructure the equation
      if let PreEquationKind::Equation {
        rhs_builder,
        fast_variable_count,
        ..
      } = &mut eq.deref_mut().pe_kind
      {
        if *fast_variable_count >= 0 {
          // Fast case
          context.substitution.clear_first_n(*fast_variable_count as usize);
          if let Some(lhs_automaton) = &mut eq2.deref_mut().lhs_automaton {
            if let (true, sp) = lhs_automaton.match_(subject, &mut context.substitution, extension_info) {
              if sp.is_some() || context.is_trace_enabled() {
                self.apply_replace_slow_case(subject, eq3, sp, context, extension_info);
              }
              
              if extension_info.is_none() || extension_info.unwrap().matched_whole() {
                rhs_builder.replace(subject, &mut context.substitution);
              }
              else {
                // ToDo: Implement `partial_replace` on `DagNodePtr`, or else determine what replaces it.
                todo!("Implement `partial_replace` on `DagNodePtr`");
                // subject.partial_replace(
                //   rhs_builder.construct(&mut context.substitution).unwrap(),
                //   extension_info,
                // );
              }
              context.equation_count += 1;
              context.finished();
              ok_to_collect_garbage();
              return true;
            }
          } else {
            unreachable!("LHS automaton expected. This is a bug.")
          }
        } else {
          // General case
          let nr_variables = eq.variable_info.protected_variable_count();
          context.substitution.clear_first_n(nr_variables as usize);
          if let Some(lhs_automaton) = &mut eq.lhs_automaton {
            if let (true, sp) = lhs_automaton.match_(
              subject,
              &mut context.substitution,
              extension_info,
            ) {
              self.apply_replace_slow_case(subject, eq, sp, context, extension_info);
            }
          }
          context.finished();
          ok_to_collect_garbage();
        }
      } else {
        unreachable!("Destructured a nonequation as an equation. This is a bug.");
      };
    }
    false
  }

  fn apply_replace_slow_case(
    &mut self,
    subject       : DagNodePtr,
    mut eq        : PreEquationPtr,
    mut sp        : MaybeSubproblem,
    context       : &mut RewritingContext,
    extension_info: MaybeExtensionInfo,
  ) -> bool {
    #[cfg(debug_assertions)]
    debug!(
      5,
      "EquationTable::applyReplace() slowCase:\nsubject = {}\neq = {}",
      subject,
      eq
    );

    if sp.is_none() || sp.as_mut().map_or(false, |s| s.solve(true, context)) {
      if !eq.has_condition() || eq.check_condition(subject, context, sp) {
        let _trace = context.is_trace_enabled();
        // ToDo: Implement tracing.
        // if trace {
        //   context.trace_pre_eq_rewrite(subject, eq, RewritingContext::NORMAL);
        //   if context.trace_abort() {
        //     context.finished();
        //     return false;
        //   }
        // }
        
        // Destructure the equation
        if let PreEquationKind::Equation { rhs_builder, .. } = &mut eq.pe_kind {
          if extension_info.is_none() || extension_info.unwrap().matched_whole() {
            rhs_builder.replace(subject, &mut context.substitution);
          } else {
            // ToDo: Implement `partial_replace` on `DagNodePtr`, or else determine what replaces it.
            //       Apparently only used by AU, ACU, S theories.
            todo!("Implement `partial_replace` on `DagNodePtr`");
            // subject.partial_replace(
            //   rhs_builder.construct(&mut context.substitution),
            //   extension_info.unwrap(),
            // );
          }
          context.equation_count += 1;
          // if trace {
          //   context.trace_post_eq_rewrite(subject.clone());
          // }
          context.finished();
          ok_to_collect_garbage();
          return true;
        }
      }
    }
    false
  }
  // endregion EquationTable methods

}

impl Display for SymbolCore {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.arity().get() {
      0u16 => write!(f, "{}", self.name),
      arity => write!(f, "{}{}", self.name, int_to_subscript(arity as u32)),
    }
  }
}

impl Formattable for SymbolCore {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    match style {
      FormatStyle::Debug => {
        write!(f, "SymbolCore<{}>", self.name)
      }

      FormatStyle::Simple
      | FormatStyle::Input
      | FormatStyle::Default => {
        write!(f, "{}", self.name)
      }
    }
  }
}
