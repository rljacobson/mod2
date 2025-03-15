/*!

Built-in data constants are in the "nonalgebraic" theory.

*/

use std::{
  fmt::Display,
  cmp::Ordering,
  any::Any,
  ops::Deref
};

use crate::{
  core::{
    symbol_core::SymbolCore,
    format::{FormatStyle, Formattable},
    term_core::TermCore
  },
  api::{
    dag_node::{DagNode, DagNodePtr},
    symbol::Symbol,
    term::{BxTerm, Term},
    built_in::{
      nonalgebraic_symbol::NASymbol,
      Float,
      Integer
    }
  },
};


pub type StringTerm  = NATerm<String>;
pub type FloatTerm   = NATerm<Float>;
pub type IntegerTerm = NATerm<Integer>;

pub struct NATerm<T: Any>{
  core     : TermCore,
  pub value: T,
}

impl<T: Any + Display> Formattable for NATerm<T> {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    let name = self.core.symbol.deref().name();
    match  style {
      FormatStyle::Debug => {
        write!(f, "{}Term<{}>", name, self.value)
      }
      
      FormatStyle::Simple
      | FormatStyle::Input
      | FormatStyle::Default => {
        write!(f, "{}", self.value)
      }
    }
  }
}

impl<T: Any + Display> Term for NATerm<T> {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn as_ptr(&self) -> *const dyn Term {
    todo!()
  }

  fn semantic_hash(&self) -> u32 {
    todo!()
  }

  fn normalize(&mut self, full: bool) -> (u32, bool) {
    todo!()
  }

  fn core(&self) -> &TermCore {
    todo!()
  }

  fn core_mut(&mut self) -> &mut TermCore {
    todo!()
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
