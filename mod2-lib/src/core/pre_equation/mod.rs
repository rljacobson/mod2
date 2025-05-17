/*!

A `PreEquation` is just a superclass for equations, rules, sort constraints, and strategies (the last of which is not
implemented.) The subclass is implemented as enum `PreEquationKind`.

*/

pub mod condition;
mod membership;

use std::fmt::{Display, Formatter};

use enumflags2::{bitflags, BitFlags};
use mod2_abs::{join_string, IString};

use crate::{core::{
  pre_equation::condition::Conditions,
  VariableInfo
}, api::{
  term::BxTerm,
  dag_node::DagNodePtr
}, impl_display_debug_for_formattable};
use crate::core::format::{FormatStyle, Formattable};
use super::sort::SortPtr;


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

/// Representation of Rule, Equation, Sort Constraint/Membership Axiom.
pub enum PreEquationKind {
  Equation {
    rhs_term: BxTerm,
    // rhs_builder:         RHSBuilder,
    fast_variable_count: i32,
    // instruction_seq    : Option<InstructionSequence>
  },

  Rule {
    rhs_term: BxTerm,
    // rhs_builder:                 RHSBuilder,
    // non_extension_lhs_automaton: Option<RcLHSAutomaton>,
    // extension_lhs_automaton:     Option<RcLHSAutomaton>,
  },

  // Membership Axiom ("Sort constraint")
  Membership {
    sort: SortPtr,
  },

  // StrategyDefinition
}

pub use PreEquationKind::*;
use crate::core::interpreter::InterpreterAttribute;

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
}

pub struct PreEquation {
  pub name      : Option<IString>,
  pub attributes: PreEquationAttributes,

  pub pe_kind      : PreEquationKind,
  pub conditions   : Conditions,
  pub lhs_term     : BxTerm,
  // pub lhs_automaton: Option<RcLHSAutomaton>,
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
      lhs_dag      : None,
      variable_info: Default::default(),
      pe_kind      : Rule{ rhs_term },
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
      lhs_dag      : None,
      variable_info: Default::default(),
      pe_kind      : Equation{ rhs_term, fast_variable_count: 0 },
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
      lhs_dag      : None,
      variable_info: Default::default(),
      pe_kind      : Membership{ sort: rhs_sort },
      index_within_parent_module: 0,
    }
  }
  
  // endregion Constructors
  
  // Common implementation
  // fn trace_begin_trial(&self, subject: RcDagNode, context: RcRewritingContext) -> Option<i32> {
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

  /*
  #[inline(always)]
  fn lhs_term(&self) -> RcTerm{
    self.lhs_term.clone()
  }
  #[inline(always)]
  fn lhs_automaton(&self) -> RcLHSAutomaton{
    self.lhs_automaton.as_ref().unwrap().clone()
  }
  #[inline(always)]
  fn lhs_dag(&self) -> RcDagNode{
    self.lhs_dag.as_ref().unwrap().clone()
  }
  #[inline(always)]
  fn condition_mut(&mut self) -> &mut Condition {
    &mut self.condition
  }
  #[inline(always)]
  pub(crate) fn variable_info(&self) -> &VariableInfo{
    &self.variable_info
  }
  #[inline(always)]
  fn variable_info_mut(&mut self) -> &mut VariableInfo{
    &mut self.variable_info
  }
  #[inline(always)]
  fn name(&self) -> Option<IString> {
    self.name.clone()
  }
  */
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

  // endregion
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