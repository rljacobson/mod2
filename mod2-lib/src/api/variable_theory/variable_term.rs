use std::{
  any::Any,
  cmp::Ordering
};
use mod2_abs::IString;
use crate::{
  core::{
    format::{FormatStyle, Formattable},
    term_core::TermCore
  },
  api::{
    variable_theory::VariableType,
    term::{Term, TermPtr},
    symbol::SymbolPtr,
    dag_node::{DagNode, DagNodePtr},
  },
  impl_display_debug_for_formattable,
};

pub struct VariableTerm {
  pub name         : IString,
  pub variable_type: VariableType,
  pub core         : TermCore,
}

impl VariableTerm {
  pub fn new(name: IString, symbol: SymbolPtr) -> Self {
    VariableTerm{
      name,
      variable_type: VariableType::Blank,
      core: TermCore::new(symbol)
    }
  }
}

impl Term for VariableTerm {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
  
  fn as_ptr(&self) -> TermPtr {
    TermPtr::new(self as *const dyn Term as *mut dyn Term)
  }

  fn hash(&self) -> u32 {
    self.symbol().hash()
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

  fn iter_args(&self) -> Box<dyn Iterator<Item=TermPtr> + '_> {
    Box::new(std::iter::empty::<TermPtr>())
  }

  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering {
    self.core.symbol.name().cmp(&other.symbol().name())
  }

  fn compare_dag_arguments(&self, other: &dyn DagNode) -> Ordering {
    self.core.symbol.name().cmp(&other.symbol().name())
  }

  fn dagify_aux(&self) -> DagNodePtr {
    todo!()
  }
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
