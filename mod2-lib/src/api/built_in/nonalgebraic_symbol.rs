use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use crate::api::built_in::{Bool, Float, Integer};
use crate::api::built_in::nonalgebraic_term::NATerm;
use crate::api::symbol::Symbol;
use crate::core::sort::SortPtr;
use crate::core::symbol_core::{SymbolAttribute, SymbolCore, SymbolType};

pub type StringSymbol  = NASymbol<String>;
pub type FloatSymbol   = NASymbol<Float>;
pub type IntegerSymbol = NASymbol<Integer>;
pub type BoolSymbol    = NASymbol<Bool>;

pub struct NASymbol<T> {
  core        : SymbolCore,
  phantom_data: PhantomData<T>,
}

impl<T> NASymbol<T>{
  pub fn new(symbol_core: SymbolCore) -> NASymbol<T>{
    NASymbol{
      core        : symbol_core,
      phantom_data: PhantomData::default()
    }
  }
}

impl<T> Symbol for  NASymbol<T>{
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
