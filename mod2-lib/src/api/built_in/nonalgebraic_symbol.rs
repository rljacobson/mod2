use std::{
  fmt::{Debug, Display},
  marker::PhantomData
};
use mod2_abs::impl_as_any_ptr_fns;
use crate::{
  api::{
    symbol::{
      SymbolPtr,
      Symbol
    },
    built_in::{
      Bool,
      Float,
      Integer,
      NaturalNumber
    },
    term::BxTerm
  },
  core::symbol::SymbolCore,
};

pub type StringSymbol  = NASymbol<String>;
pub type FloatSymbol   = NASymbol<Float>;
pub type IntegerSymbol = NASymbol<Integer>;
pub type NaturalSymbol = NASymbol<NaturalNumber>;
pub type BoolSymbol    = NASymbol<Bool>;
pub type NaturalNumberSymbol = NaturalSymbol;

pub struct NASymbol<T> {
  core        : SymbolCore,
  phantom_data: PhantomData<T>,
}

impl<T: 'static> NASymbol<T>{
  pub fn new(symbol_core: SymbolCore) -> NASymbol<T>{
    NASymbol{
      core        : symbol_core,
      phantom_data: PhantomData::default()
    }
  }
}

impl<T: 'static> Symbol for  NASymbol<T>{
  // impl_as_any_ptr_fns!(Symbol, NASymbol);
  fn as_any(&self) -> &dyn std::any::Any { self }
  fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
  fn as_ptr(&self) -> SymbolPtr {
    SymbolPtr::new(self as *const dyn Symbol as *mut dyn Symbol)
  }

  fn make_term(&self, _args: Vec<BxTerm>) -> BxTerm {
    // Use NATerm<T>::new(value: T) instead.
    unimplemented!()
  }

  fn core(&self) -> &SymbolCore {
    &self.core
  }

  fn core_mut(&mut self) -> &mut SymbolCore {
    &mut self.core
  }
}

// region Impls

impl<T: Display> Display for NASymbol<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.core.name)
  }
}

impl<T: Debug> Debug for NASymbol<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}Symbol<{}>", std::any::type_name::<T>(), self.core.name)
  }
}

// endregion
