use std::{
  any::Any,
  cmp::Ordering
};

use crate::{
  core::{
    format::{FormatStyle, Formattable},
    term_core::TermCore
  },
  api::{
    variable_theory::VariableType,
    term::Term,
    symbol::SymbolPtr,
    dag_node::{DagNode, DagNodePtr}
  },
  impl_display_debug_for_formattable
};

pub struct VariableTerm {
  pub core         : TermCore,
  pub variable_type: VariableType,
}

impl Formattable for VariableTerm {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    let symbol: SymbolPtr = self.symbol();

    symbol.repr(f, style)?;

    match style {
      FormatStyle::Default
      | FormatStyle::Simple
      | FormatStyle::Input => {
        match self.variable_type {
          VariableType::Blank        => write!(f, "_"),
          VariableType::Sequence     => write!(f, "__"),
          VariableType::NullSequence => write!(f, "___"),
        }
      }

      FormatStyle::Debug => {
        match self.variable_type {
          VariableType::Blank        => write!(f, "<Blank>"),
          VariableType::Sequence     => write!(f, "<Sequence>"),
          VariableType::NullSequence => write!(f, "<NullSequence>"),
        }
      }
    }

  }
}
impl_display_debug_for_formattable!(VariableTerm);


impl Term for VariableTerm {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn as_ptr(&self) -> *const dyn Term {
    self as *const dyn Term
  }

  fn semantic_hash(&self) -> u32 {
    todo!()
  }

  fn normalize(&mut self, _full: bool) -> (u32, bool) {
    todo!()
  }

  fn core(&self) -> &TermCore {
    &self.core
  }

  fn core_mut(&mut self) -> &mut TermCore {
    &mut self.core
  }

  fn iter_args(&self) -> Box<dyn Iterator<Item=&dyn Term> + '_> {
    todo!()
  }

  fn compare_term_arguments(&self, _other: &dyn Term) -> Ordering {
    todo!()
  }

  fn compare_dag_arguments(&self, _other: &dyn DagNode) -> Ordering {
    todo!()
  }

  fn dagify_aux(&self) -> DagNodePtr {
    todo!()
  }
}