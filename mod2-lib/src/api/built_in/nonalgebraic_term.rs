/*!

Built-in data constants are in the "nonalgebraic" theory.

*/

use std::{
  any::Any,
  cmp::Ordering,
  fmt::Display,
  hash::{Hash, Hasher},
  mem::transmute,
  ops::{
    Deref,
    BitAnd,
    BitXor,
  },
  str::FromStr,
};
use ordered_float::OrderedFloat;
use mod2_abs::{
  hash::{hash2, FastHasher},
  NatSet
};
use crate::{
  api::{
    built_in::{
      Bool,
      Float,
      Integer,
      NADataType,
      NADagNode,
      NaturalNumber,
      StringBuiltIn,
      get_built_in_symbol,
    },
    dag_node::{DagNode, DagNodePtr},
    symbol::Symbol,
    term::{Term, TermPtr, BxTerm},
  },
  core::{
    format::{FormatStyle, Formattable},
    term_core::TermCore,
    TermBag,
  },
  HashType,
};
use crate::api::built_in::FloatDagNode;
use crate::api::dag_node_cache::DagNodeCache;

pub type BoolTerm    = NATerm<Bool>;
pub type FloatTerm   = NATerm<Float>;
pub type IntegerTerm = NATerm<Integer>;
pub type StringTerm  = NATerm<StringBuiltIn>;
pub type NaturalTerm = NATerm<NaturalNumber>;
pub type NaturalNumberTerm = NATerm<NaturalNumber>;

#[derive(Clone)]
pub struct NATerm<T: NADataType>{
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

  pub fn from_str(x: &str) -> Self {
    Self::new(x)
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

  pub fn from_str(x: &str) -> Self {
    let value: Float = match x.parse(){
      Ok(x) => x,
      Err(_) => {
        panic!("could not parse {}", x);
      }
    };
    Self::new(value)
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

  pub fn from_str(x: &str) -> Self {
    let value: Integer = match x.parse(){
      Ok(x) => x,
      Err(_) => {
        panic!("could not parse {}", x);
      }
    };
    Self::new(value)
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

  pub fn from_str(x: &str) -> Self {
    let value: NaturalNumber = match x.parse(){
      Ok(x) => x,
      Err(_) => {
        panic!("could not parse {}", x);
      }
    };
    Self::new(value)
  }
}

impl BoolTerm {
  pub fn new(value: Bool) -> BoolTerm {
    let symbol = match value {
      true => unsafe{get_built_in_symbol("true").unwrap_unchecked()},
      false => unsafe{get_built_in_symbol("false").unwrap_unchecked()},
    };
    let core = TermCore::new(symbol);
    BoolTerm {
      core,
      value: value.into(),
    }
  }

  pub fn from_str(x: &str) -> Self {
    let value: bool = match x.parse(){
      Ok(x) => x,
      Err(_) => {
        panic!("could not parse {}", x);
      }
    };
    Self::new(value)
  }
}

impl<T: NADataType> Formattable for NATerm<T> {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    let name = self.core.symbol.name();
    let value_str = if *name == *"String" {
      format!("\"{}\"", self.value)
    } else {
      self.value.to_string()
    };
    match  style {
      FormatStyle::Debug => {
        write!(f, "{}Term<{}>", name, value_str)
      }

      FormatStyle::Simple
      | FormatStyle::Input
      | FormatStyle::Default => {
        write!(f, "{}", value_str)
      }
    }
  }
}

impl<T: NADataType> Term for NATerm<T> {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn as_ptr(&self) -> TermPtr {
    TermPtr::new(self as *const dyn Term as *mut dyn Term)
  }

  fn copy(&self) -> BxTerm {
    Box::new(self.clone())
  }

  fn normalize(&mut self, _full: bool) -> (Option<BxTerm>, bool, HashType) {
    let hash_value = hash2(self.symbol().hash(), self.value.hashable_bits());
    self.core_mut().hash_value = hash_value;

    (None, false, hash_value)
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
    let other: &NATerm<T> = other
        .as_any()
        .downcast_ref::<NATerm<T>>()
        .expect("NATerm type mismatch: cannot compare");
    
    self.value.compare(&other.value)
  }

  fn compare_dag_arguments(&self, other: DagNodePtr) -> Ordering {
    if let Some(other) = other.as_any().downcast_ref::<NADagNode<T>>() {
      self.value.compare(&T::value_from_dag_node(other))
    } else { 
      panic!("NATerm type mismatch: cannot compare");
    }
  }

  #[allow(private_interfaces)]
  fn dagify_aux(&self, _node_cache: &mut DagNodeCache) -> DagNodePtr {
    T::make_dag_node(self.value.clone())
  }

  fn analyse_constraint_propagation(&mut self, _bound_uniquely: &mut NatSet) {
    /* nothing to do */
  }

  fn find_available_terms_aux(&self, _available_terms: &mut TermBag, _eager_context: bool, _at_top: bool) {
    /* nothing to do */
  }
}
