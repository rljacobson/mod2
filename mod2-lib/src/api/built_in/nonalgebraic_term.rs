/*!

Built-in data constants are in the "nonalgebraic" theory.

*/

use std::{
  any::Any,
  cmp::Ordering,
  fmt::Display,
  ops::Deref
};

use crate::{
  api::{
    built_in::{
      Float,
      Integer,
      NaturalNumber,
      StringBuiltIn,
      get_built_in_symbol, 
    },
    dag_node::{DagNode, DagNodePtr},
    symbol::Symbol,
    term::Term
  },
  core::{
    format::{FormatStyle, Formattable},
    term_core::TermCore
  },
};

pub type StringTerm  = NATerm<StringBuiltIn>;
pub type FloatTerm   = NATerm<Float>;
pub type IntegerTerm = NATerm<Integer>;
pub type NaturalNumberTerm = NATerm<NaturalNumber>;

pub struct NATerm<T: Any>{
  core     : TermCore,
  pub value: T,
}

impl StringTerm {
  pub fn new(value: &str) -> StringTerm {
    let core = TermCore::new(unsafe{get_built_in_symbol("String").unwrap_unchecked()});
    StringTerm {
      core,
      value: value.into(),
    }
  }
}

impl FloatTerm {
  pub fn new(value: Float) -> FloatTerm {
    let core = TermCore::new(unsafe{get_built_in_symbol("Float").unwrap_unchecked()});
    FloatTerm {
      core,
      value: value.into(),
    }
  }
}

impl IntegerTerm {
  pub fn new(value: Integer) -> IntegerTerm {
    let core = TermCore::new(unsafe{get_built_in_symbol("Integer").unwrap_unchecked()});
    IntegerTerm {
      core,
      value: value.into(),
    }
  }
}

impl NaturalNumberTerm {
  pub fn new(value: NaturalNumber) -> NaturalNumberTerm {
    let core = TermCore::new(unsafe{get_built_in_symbol("NaturalNumber").unwrap_unchecked()});
    NaturalNumberTerm {
      core,
      value: value.into(),
    }
  }
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

  fn normalize(&mut self, _full: bool) -> (u32, bool) {
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
