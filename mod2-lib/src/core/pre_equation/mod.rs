/*!

A `PreEquation` is just a superclass for equations, rules, sort constraints, and strategies (the last of which is not
implemented.) The subclass is implemented as enum `PreEquationKind`.

*/

pub mod condition;
// mod membership;
mod sort_constraint_table;
mod state_transition_graph;

use std::fmt::{Display, Formatter};

use enumflags2::{bitflags, BitFlags};
use mod2_abs::{join_string, warning, IString, NatSet, UnsafePtr};

use crate::{
  api::{
  BxLHSAutomaton,
  DagNodePtr,
  MaybeSubproblem,
  BxTerm
},
  core::{
  format::{FormatStyle, Formattable},
  gc::ok_to_collect_garbage,
  interpreter::InterpreterAttribute,
  pre_equation::{
    condition::ConditionState,
    condition::Conditions
  },
  automata::RHSBuilder,
  rewriting_context::RewritingContext,
  VariableInfo,
  TermBag,
  VariableIndex,
},
  impl_display_debug_for_formattable,
  NONE
};
use super::sort::SortPtr;
pub use sort_constraint_table::SortConstraintTable;


pub type BxPreEquation  = Box<PreEquation>;
pub type PreEquationPtr = UnsafePtr<PreEquation>;


#[bitflags]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PreEquationAttribute {
  Compiled,     // PreEquation
  NonExecute,   // PreEquation
  Otherwise,    // Equation, "owise"
  Variant,      // Equation
  Print,        // StatementAttributeInfo--not a `PreEquation`
  Narrowing,    // Rule
  Bad,          // A malformed pre-equation
}
pub type PreEquationAttributes = BitFlags<PreEquationAttribute>;

impl Display for PreEquationAttribute {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      PreEquationAttribute::Compiled   => write!(f, "compiled"),
      PreEquationAttribute::NonExecute => write!(f, "nonexecute"),
      PreEquationAttribute::Otherwise  => write!(f, "otherwise"),
      PreEquationAttribute::Variant    => write!(f, "variant"),
      PreEquationAttribute::Print      => write!(f, "print"),
      PreEquationAttribute::Narrowing  => write!(f, "narrowing"),
      PreEquationAttribute::Bad        => write!(f, "bad"),
    }
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PreEquationKindLabel {
  Equation,
  Rule,
  // Membership Axiom ("Sort constraint")
  Membership,
  // StrategyDefinition
}

impl PreEquationKindLabel {
  // Alias for Membership
  #![allow(non_upper_case_globals)]
  const SortConstraint: PreEquationKindLabel = PreEquationKindLabel::Membership;
}

/// Representation of Rule, Equation, Sort Constraint/Membership Axiom.
pub enum PreEquationKind {
  Equation {
    rhs_term           : BxTerm,
    rhs_builder        : RHSBuilder,
    fast_variable_count: i32,
    // instruction_seq    : Option<InstructionSequence>
  },

  Rule {
    rhs_term                   : BxTerm,
    rhs_builder                : RHSBuilder,
    non_extension_lhs_automaton: Option<BxLHSAutomaton>,
    extension_lhs_automaton    : Option<BxLHSAutomaton>,
  },

  // Membership Axiom ("Sort constraint")
  Membership {
    sort: SortPtr,
  },

  // StrategyDefinition
}

pub use PreEquationKind::*;

impl PreEquationKind {
  pub fn noun(&self) -> &'static str {
    match self {
      Equation { .. } => "equation",
      Rule { .. } => "rule",
      Membership { .. } => "sort constraint",
      // StrategyDefinition { .. } => "strategy definition",
    }
  }

  pub fn interpreter_trace_attribute(&self) -> InterpreterAttribute {
    match &self {
      Equation { .. } => InterpreterAttribute::TraceEq,
      Rule { .. } => InterpreterAttribute::TraceRl,
      Membership { .. } => InterpreterAttribute::TraceMb,
      // StrategyDefinition { .. } => InterpreterAttribute::TraceSd,
    }
  }

  pub fn label(&self) -> PreEquationKindLabel {
    match self {
      Equation { .. } => PreEquationKindLabel::Equation,
      Rule { .. } => PreEquationKindLabel::Rule,
      Membership { .. } => PreEquationKindLabel::Membership,
    }
  }
}

pub struct PreEquation {
  pub name         : Option<IString>,
  pub attributes   : PreEquationAttributes,
  pub pe_kind      : PreEquationKind,
  pub conditions   : Conditions,
  pub lhs_term     : BxTerm,
  pub lhs_automaton: Option<BxLHSAutomaton>,
  pub lhs_dag      : Option<DagNodePtr>,
  pub variable_info: VariableInfo,

  // `ModuleItem`
  pub index_within_parent_module: i32,
  // pub parent_module             : WeakModule,
}

impl PreEquation {
  // region Constructors
  // Several fields of a new `PreEquation` will be empty/default upon first creation.

  pub fn new_rule(
    name      : Option<IString>,
    lhs_term  : BxTerm,
    rhs_term  : BxTerm,
    conditions: Conditions,
    attributes: PreEquationAttributes,
  ) -> Self {
    PreEquation{
      name,
      attributes,
      conditions,
      lhs_term,
      lhs_automaton: None,
      lhs_dag      : None,
      variable_info: Default::default(),
      pe_kind      : Rule{
        rhs_term,
        rhs_builder                : Default::default(),
        non_extension_lhs_automaton: None,
        extension_lhs_automaton    : None
      },
      index_within_parent_module: 0,
    }
  }

  pub fn new_equation(
    name      : Option<IString>,
    lhs_term  : BxTerm,
    rhs_term  : BxTerm,
    conditions: Conditions,
    attributes: PreEquationAttributes,
  ) -> Self {
    PreEquation{
      name,
      attributes,
      conditions,
      lhs_term,
      lhs_automaton: None,
      lhs_dag      : None,
      variable_info: Default::default(),
      pe_kind      : Equation{
        rhs_term,
        rhs_builder        : Default::default(),
        fast_variable_count: 0
      },
      index_within_parent_module: 0,
    }
  }

  pub fn new_membership(
    name      : Option<IString>,
    lhs_term  : BxTerm,
    rhs_sort  : SortPtr,
    conditions: Conditions,
    attributes: PreEquationAttributes,
  ) -> Self {
    PreEquation{
      name,
      attributes,
      conditions,
      lhs_term,
      lhs_automaton: None,
      lhs_dag      : None,
      variable_info: Default::default(),
      pe_kind      : Membership{ sort: rhs_sort },
      index_within_parent_module: 0,
    }
  }

  // endregion Constructors

  pub fn as_ptr(&self) -> PreEquationPtr {
    PreEquationPtr::new(self as *const PreEquation as *mut PreEquation)
  }

  // Common implementation
  // fn trace_begin_trial(&self, subject: DagNodePtr, context: RcRewritingContext) -> Option<i32> {
  //   context.borrow_mut().trace_begin_trial(subject, self)
  // }

  // region Accessors
  #[inline(always)]
  pub(crate) fn condition(&self) -> &Conditions {
    &self.conditions
  }

  #[inline(always)]
  pub(crate) fn get_index_within_module(&self) -> i32 {
    self.index_within_parent_module
  }

  //endregion

  // region  Attributes
  #[inline(always)]
  pub(crate) fn has_condition(&self) -> bool {
    // ToDo: Can we not just check for empty?
    self.conditions.is_empty()
  }

  #[inline(always)]
  fn is_nonexec(&self) -> bool {
    self.attributes.contains(PreEquationAttribute::NonExecute)
  }

  #[inline(always)]
  fn is_compiled(&self) -> bool {
    self.attributes.contains(PreEquationAttribute::Compiled)
  }

  #[inline(always)]
  fn is_variant(&self) -> bool {
    self.attributes.contains(PreEquationAttribute::Variant)
  }

  #[inline(always)]
  fn set_nonexec(&mut self) {
    self.attributes.insert(PreEquationAttribute::NonExecute);
  }

  #[inline(always)]
  fn set_variant(&mut self) {
    self.attributes.insert(PreEquationAttribute::Variant);
  }

  #[inline(always)]
  pub fn is_narrowing(&self) -> bool {
    self.attributes.contains(PreEquationAttribute::Narrowing)
  }

  #[inline(always)]
  pub fn is_owise(&self) -> bool {
    self.attributes.contains(PreEquationAttribute::Otherwise)
  }

  // endregion Attributes

  // region Compiler related methods

  pub fn compile(&mut self, mut compile_lhs: bool) {
    if self.attributes.contains(PreEquationAttribute::Compiled) {
      return;
    }
    self.attributes.insert(PreEquationAttribute::Compiled);

    let mut available_terms = TermBag::default(); // terms available for reuse

    let kind = self.pe_kind.label();
    match kind {

      PreEquationKindLabel::Equation => {
        self.compile_build(&mut available_terms, true);


        if self.is_variant() {
          // If the equation has the variant attribute, we disallow left->right sharing so
          // that the rhs can still be instantiated, even if the substitution was made by
          // unification.
          let Equation { rhs_term, rhs_builder, .. } = &mut self.pe_kind else { unreachable!() };
          let mut dummy = TermBag::new();
          rhs_term.compile_top_rhs(rhs_builder, &mut self.variable_info, &mut dummy);
          // For an equation with the variant attribute we always compile the lhs, even if the parent symbol
          // doesn't make use of the compiled lhs (in the free theory because it uses a discrimination
          // net for lhs matching).
          compile_lhs = true;
        } else {
          // normal case
          let Equation { rhs_term, rhs_builder, .. } = &mut self.pe_kind else { unreachable!() };
          rhs_term.compile_top_rhs(rhs_builder, &mut self.variable_info, &mut available_terms);
        }

        self.compile_match(compile_lhs, true);
        let Equation { rhs_builder, fast_variable_count, .. } = &mut self.pe_kind else { unreachable!() };
        rhs_builder.remap_indices(&mut self.variable_info);
        *fast_variable_count = if self.conditions.is_empty() {
          NONE
        } else {
          self.variable_info.protected_variable_count() as i32
        }; // HACK
      } // end Equation

      PreEquationKindLabel::Rule => {
        // Since rules can be applied in non-eager subterms, if we have
        // a condition we must consider all variables to be non-eager
        // to avoid having a condition reduce a lazy subterm.
        self.compile_build(&mut available_terms, !self.has_condition());


        // HACK: we pessimize the compilation of unconditional rules to avoid
        // left->right subterm sharing that would break narrowing.
        if !self.has_condition() {
          let Rule { rhs_term, rhs_builder, .. } = &mut self.pe_kind else { unreachable!() };
          let mut dummy = TermBag::new();
          rhs_term.compile_top_rhs(rhs_builder, &mut self.variable_info, &mut dummy);
        } else {
          let Rule { rhs_term, rhs_builder, .. } = &mut self.pe_kind else { unreachable!() };
          rhs_term.compile_top_rhs(rhs_builder, &mut self.variable_info, &mut available_terms);
        }

        self.compile_match(compile_lhs, true);
        let Rule { rhs_builder, .. } = &mut self.pe_kind else { unreachable!() };
        rhs_builder.remap_indices(&mut self.variable_info);

        // Make all variables in a rules lhs into condition variables so that
        // if we compile lhs again in get_non_ext_lhs_automaton() or get_ext_lhs_automaton()
        // it will be compiled to generate all matchers rather than just those
        // that differ on variables in the condition.
        self.variable_info
            .add_condition_variables(self.lhs_term.occurs_below());
      } // end Rule

      // Membership and StrategyDefinition
      _ => {
        self.compile_build(&mut available_terms, false);
        self.compile_match(compile_lhs, false);
      }

    }
  }


  fn compile_build(&mut self, available_terms: &mut TermBag, eager_context: bool) {
    // Fill the hash set of terms for structural sharing
    self.lhs_term.find_available_terms(available_terms, eager_context, true);
    {
      // Scope of `lhs_term`
      self.lhs_term.determine_context_variables();
      self.lhs_term.insert_abstraction_variables(&mut self.variable_info);
    }

    let fragment_count = self.conditions.len();
    for i in 0..fragment_count {
      self.conditions[i].as_mut().compile_build(&mut self.variable_info, available_terms);
    }
  }

  fn compile_match(&mut self, compile_lhs: bool, with_extension: bool) {
    let lhs_term = self.lhs_term.as_mut();

    let required_substitution_size = self.variable_info.compute_index_remapping();
    // We don't assume that our module was set, so we look at the module of the lhs top symbol.
    lhs_term.symbol()
            .core_mut()
            .parent_module
            .as_mut()
            .unwrap()
            .notify_substitution_size(required_substitution_size);

    if compile_lhs {
      let mut bound_uniquely = NatSet::new();

      let (lhs_automaton, _) = lhs_term.compile_lhs(with_extension, &mut self.variable_info, &mut bound_uniquely);
      self.lhs_automaton     = Some(lhs_automaton);
    }

    {
      // Scope of variable_info
      let fragment_count = self.conditions.len();
      for i in 0..fragment_count {
        self.conditions[i].as_mut().compile_match(&mut self.variable_info, lhs_term.occurs_below_mut());
      }
    }
  }

  // endregion Compiler related methods

  // region `check*` methods

  /// Normalize lhs and recursively collect the indices and occurs sets of this term and its descendants
  fn check(&mut self) {
    self.lhs_term.normalize(true);
    self.lhs_term.index_variables(&mut self.variable_info);

    let mut bound_variables: NatSet = self.lhs_term.occurs_below().clone(); // Deep copy

    for condition_fragment in self.conditions.iter_mut() {
      condition_fragment.check(&mut self.variable_info, &mut bound_variables);
    }

    match &mut self.pe_kind {
      Equation { rhs_term, .. } => {
        rhs_term.normalize(false);
        rhs_term.index_variables(&mut self.variable_info);

        let unbound_variables = rhs_term.occurs_below_mut();
        unbound_variables.difference_in_place(&bound_variables);
        self.variable_info.add_unbound_variables(unbound_variables);
      }

      Rule { rhs_term, .. } => {
        rhs_term.normalize(false);
        rhs_term.index_variables(&mut self.variable_info);

        let unbound_variables = rhs_term.occurs_below().difference(&bound_variables);
        self.variable_info.add_unbound_variables(&unbound_variables);

        if !self.is_nonexec() && !self.variable_info.unbound_variables.is_empty() {
          let mindex       = VariableIndex::from_usize(self.variable_info.unbound_variables.min_value().unwrap());
          let min_variable = self.variable_info.index_to_variable(mindex).unwrap();

          let mut self_string_simple:  String = "".to_string();
          let mut self_string_default: String = "".to_string();
          self.repr(&mut self_string_simple,  FormatStyle::Simple).unwrap();
          self.repr(&mut self_string_default, FormatStyle::Default).unwrap();
          warning!(
            1,
            "{}: variable {} is used before it is bound in {}:\n{}",
            self_string_simple,
            min_variable,
            self.pe_kind.noun(),
            self_string_default
          );

          // Rules with variables used before they are bound have a legitimate purpose - they can be used with metaApply()
          // and a substitution. So we just make the rule nonexec rather than marking it as bad.

          self.set_nonexec();
        }
      }

      Membership { .. } => {
        // Doesn't use bound_variables.
        if !self.is_nonexec() && !self.variable_info.unbound_variables.is_empty() {
          let mindex       = VariableIndex::from_usize(self.variable_info.unbound_variables.min_value().unwrap());
          let min_variable = self.variable_info.index_to_variable(mindex).unwrap();

          let mut self_string_simple  = String::new();
          let mut self_string_default = String::new();
          self.repr(&mut self_string_simple,  FormatStyle::Simple).unwrap();
          self.repr(&mut self_string_default, FormatStyle::Default).unwrap();
          warning!(
            1,
            "{}: variable {} is used before it is bound in {}:\n{}",
            self_string_simple,
            min_variable,
            self.pe_kind.noun(),
            self_string_default
          );

          // No legitimate use for such sort constraints so mark it as bad.
          self.attributes.insert(PreEquationAttribute::Bad);
        }

      }
    }
  }

  ///  This is the most general condition checking function that allows multiple distinct successes; caller must provide
  ///  trial_ref variable and condition state stack in order to preserve this information between calls.
  pub(crate) fn check_condition_find_first(
    &mut self,
    mut find_first: bool,
    _subject      : DagNodePtr, // Used only for tracing
    context       : &mut RewritingContext,
    mut subproblem: MaybeSubproblem,
    trial_ref     : &mut Option<i32>,
    state         : &mut Vec<ConditionState>,
  ) -> bool
  {
    assert_ne!(self.conditions.len(), 0, "no condition");
    assert!(!find_first || state.is_empty(), "non-empty condition state stack");

    if find_first {
      *trial_ref = None;
    }

    loop {
      // ToDo: Implement trace status
      // if trace_status() {
      //   if find_first {
      //     *trial_ref = self.trace_begin_trial(subject.clone(), context.clone());
      //   }
      //   if context.borrow().trace_abort() {
      //     state.clear();
      //     // return false since condition variables may be unbound
      //     return false;
      //   }
      // }

      // todo!("Uncomment the following line.");
      let success: bool = self.solve_condition(find_first, trial_ref, context, state);
      // let success = true;

      // if trace_status() {
      //   if context.borrow().trace_abort() {
      //     state.clear();
      //     return false; // return false since condition variables may be unbound
      //   }
      //
      //   context.borrow_mut().trace_end_trial(*trial_ref, success);
      // }

      if success {
        return true;
      }
      assert!(state.is_empty(), "non-empty condition state stack");
      find_first = true;
      *trial_ref = None;

      // Condition evaluation may create nodes without doing rewrites so run GC safe point.
      ok_to_collect_garbage();
      if let Some(subproblem) = &mut subproblem {
        if !subproblem.solve(false, context) {
          break;
        }
      } else {
        break;
      }
    }
    // if trace_status() && trial_ref.is_some() {
    //   context.borrow().trace_exhausted(*trial_ref);
    // }
    false
  }

  /// Simplified interface to `check_condition_find_first(…)` for the common case where we only care
  /// if a condition succeeds at least once or fails.
  pub(crate) fn check_condition(
    &mut self,
    subject   : DagNodePtr,
    context   : &mut RewritingContext,
    subproblem: MaybeSubproblem,
  ) -> bool
  {
    let mut trial_ref: Option<i32>         = None;
    let mut state    : Vec<ConditionState> = Vec::new();

    let result = self.check_condition_find_first(true, subject, context, subproblem, &mut trial_ref, &mut state);

    assert!(result || state.is_empty(), "non-empty condition state stack");
    // state drops its elements when it goes out of scope.
    // state.clear();

    result
  }

  fn solve_condition(
    &mut self,
    mut find_first: bool,
    _trial_ref    : &mut Option<i32>,
    solution      : &mut RewritingContext,
    state         : &mut Vec<ConditionState>,
  ) -> bool {
    let fragment_count = self.conditions.len();
    let mut i = if find_first {
      0
    } else {
      fragment_count - 1
    };

    loop {
      // if trace_status() {
      //   if solution.borrow().trace_abort() {
      //     return false; // ToDo: This doesn't look right.
      //   }
      //   solution.borrow().trace_begin_fragment(*trial_ref, self.conditions[i].as_ref(), find_first);
      // }

      // A cute way to do backtracking.
      find_first = self.conditions[i].solve(find_first, solution, state);

      // if trace_status() {
      //   if solution.borrow().trace_abort() {
      //     return false; // ToDo: This doesn't look right.
      //   }
      //   solution.borrow_mut().trace_end_fragment(
      //     *trial_ref, self, //.condition[i].as_ref(),
      //     i, find_first,
      //   );
      // }

      if find_first {
        if i == fragment_count - 1 {
          break;
        }
        i += 1;
      } else {
        if i == 0 {
          break;
        }
        i -= 1;
      }
    }

    find_first
  }

  // endregion `check*` methods

}

impl Formattable for PreEquation {
  fn repr(&self, f: &mut dyn std::fmt::Write, _style: FormatStyle) -> std::fmt::Result {
    match &self.pe_kind {

      PreEquationKind::Equation { rhs_term, .. } => {
        write!(f, "equation {} = {}", self.lhs_term,  rhs_term)?;
      }

      PreEquationKind::Rule { rhs_term, .. } => {
        write!(f, "rule {} => {}", self.lhs_term,  rhs_term)?;
      }

      PreEquationKind::Membership { sort } => {
        write!(f, "membership {} : {}", self.lhs_term,  sort)?;
      }

    }

    // conditions
    if !self.conditions.is_empty() {
      write!(
        f,
        " if {}",
        join_string(self.conditions.iter(), " ⋀ ")
      )?;
    }

    // attributes
    if !self.attributes.is_empty() {
      write!(
        f,
        " [{}]",
        join_string(self.attributes.iter(), ", ")
      )?;
    }

    write!(f, ";")
  }
}

impl_display_debug_for_formattable!(PreEquation);
