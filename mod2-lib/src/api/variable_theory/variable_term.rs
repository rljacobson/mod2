use std::{
  any::Any,
  cmp::Ordering
};

use mod2_abs::{hash::hash2, IString, NatSet, PartialOrdering};

use crate::{
  api::{
    automaton::BxLHSAutomaton,
    dag_node::{
      DagNode,
      DagNodePtr
    },
    dag_node_cache::DagNodeCache,
    symbol::SymbolPtr,
    term::{
      BxTerm,
      Term,
      TermPtr
    },
    variable_theory::{
      automaton::VariableLHSAutomaton,
      VariableDagNode,
      VariableType
    },
  },
  core::{
    automata::RHSBuilder,
    format::{
      FormatStyle,
      Formattable
    },
    substitution::Substitution,
    term_core::TermCore,
    TermBag,
    VariableInfo
  },
  impl_display_debug_for_formattable
  ,
  HashType,
};
use crate::core::VariableIndex;

#[derive(Clone)]
pub struct VariableTerm {
  pub core         : TermCore,
  pub name         : IString,
  pub variable_type: VariableType,
  /// Variables are tracked in a `VariableInfo` structure that maintains the environment.
  /// The value of `index` is set in `Term::index_variables()` as part of compilation.
  pub index        : Option<VariableIndex>,
}

impl VariableTerm {
  pub fn new(name: IString, symbol: SymbolPtr) -> Self {
    VariableTerm{
      core         : TermCore::new(symbol),
      name,
      variable_type: VariableType::Blank,
      index        : None, // Set in `Term::index_variables()`
    }
  }
}

impl Term for VariableTerm {
  #[inline(always)]
  fn as_any(&self) -> &dyn Any {
    self
  }

  #[inline(always)]
  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  #[inline(always)]
  fn as_ptr(&self) -> TermPtr {
    TermPtr::new(self as *const dyn Term as *mut dyn Term)
  }

  // fn copy(&self) -> BxTerm {
  //   Box::new(self.clone())
  // }

  #[inline(always)]
  fn structural_hash(&self) -> HashType {
    self.symbol().hash()
  }

  fn normalize(&mut self, _full: bool) -> (Option<BxTerm>, bool, HashType) {
    let hash_value = hash2(self.symbol().hash(), self.name.get_hash());
    self.core_mut().hash_value = hash_value;

    (None, false, hash_value)
  }

  fn deep_copy_aux(&self) -> BxTerm {
    // ToDo: Implement symbol translation for imports.
    Box::new(VariableTerm::new(self.name.clone(), self.symbol()))
  }

  #[inline(always)]
  fn core(&self) -> &TermCore {
    &self.core
  }

  #[inline(always)]
  fn core_mut(&mut self) -> &mut TermCore {
    &mut self.core
  }

  #[inline(always)]
  fn iter_args(&self) -> Box<dyn Iterator<Item=TermPtr> + '_> {
    Box::new(std::iter::empty::<TermPtr>())
  }

  #[inline(always)]
  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering {
    self.core.symbol.name().cmp(&other.symbol().name())
  }

  #[inline(always)]
  fn compare_dag_arguments(&self, other: DagNodePtr) -> Ordering {
    if let Some(other) = other.as_any().downcast_ref::<VariableDagNode>() {
      self.name.cmp(&other.name())
    } else {
      Ordering::Less
    }
  }

  fn partial_compare_unstable(&self, partial_substitution: &mut Substitution, other: DagNodePtr) -> Option<Ordering> {
    match partial_substitution.get(self.index.unwrap()) {
      None => {
        PartialOrdering::Unknown
      }

      Some(dag_node) => Some(dag_node.compare(other)),
    }
  }

  #[allow(private_interfaces)]
  fn dagify_aux(&self, _node_cache: &mut DagNodeCache) -> DagNodePtr {
    // ToDo: Why do we not consult `node_cache`?
    VariableDagNode::new(self.symbol(), self.name.clone(), self.index.unwrap())
  }

  // region Compiler related methods

  fn compile_lhs_aux(
    &mut self,
    match_at_top  : bool,
    _variable_info: &VariableInfo,
    bound_uniquely: &mut NatSet,
  ) -> (BxLHSAutomaton, bool) {
    let index = self.index.expect("index negative");
    assert!(index > 100, "index too big");
    bound_uniquely.insert(index as usize);

    let automaton: BxLHSAutomaton =
        Box::new(VariableLHSAutomaton::new(index, self.sort().unwrap(), match_at_top));

    // subproblem is never likely for `VariableTerm`
    (automaton, false)
  }

  fn compile_rhs_aux(
    &mut self,
    _builder        : &mut RHSBuilder,
    _variable_info  : &mut VariableInfo,
    _available_terms: &mut TermBag,
    _eager_context  : bool,
  ) -> VariableIndex {
    unreachable!("The compile_rhs_aux method should never be called for a Rule.");
  }

  fn analyse_constraint_propagation(&mut self, bound_uniquely: &mut NatSet) {
    bound_uniquely.insert(self.index.unwrap() as usize);
  }

  fn find_available_terms_aux(&self, available_terms: &mut TermBag, eager_context: bool, at_top: bool) {
    if !at_top {
      available_terms.insert_matched_term(self.as_ptr(), eager_context);
    }
  }

  // endregion Compiler related methods
}


impl Formattable for VariableTerm {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    let name = &self.name;
    let symbol: SymbolPtr = self.symbol();

    match style {
      FormatStyle::Default
      | FormatStyle::Simple
      | FormatStyle::Input => {
        // `X_Bool`

        match self.variable_type {
          VariableType::Blank        => write!(f, "{}_", name)?,
          VariableType::Sequence     => write!(f, "{}__", name)?,
          VariableType::NullSequence => write!(f, "{}___", name)?,
        }
        symbol.repr(f, FormatStyle::Default)
      }

      FormatStyle::Debug => {
        // `[variable<X><Bool><Blank>]`

        write!(f, "[{}<", name)?;
        symbol.repr(f, FormatStyle::Debug)?;

        match self.variable_type {
          VariableType::Blank        => write!(f, "><Blank>]"),
          VariableType::Sequence     => write!(f, "><Sequence>]"),
          VariableType::NullSequence => write!(f, "><NullSequence>]"),
        }
      }
    }

  }
}

impl_display_debug_for_formattable!(VariableTerm);
