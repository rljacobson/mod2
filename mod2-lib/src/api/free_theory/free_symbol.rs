use mod2_abs::IString;
use crate::{
  api::{
    symbol::Symbol,
    Arity
  },
  core::format::{FormatStyle, Formattable},
  impl_display_debug_for_formattable
};
use crate::core::sort::SortPtr;
use crate::core::symbol::{SymbolAttributes, SymbolType};
use crate::core::symbol::SymbolCore;

pub struct FreeSymbol {
  core: SymbolCore
}

impl FreeSymbol {
  pub fn new(
    name: IString,
    arity: Arity,
    attributes : SymbolAttributes,
    symbol_type: SymbolType,
  ) -> FreeSymbol {
    let core = SymbolCore::new(name, arity, attributes, symbol_type);
    FreeSymbol{ core }
  }
  
  pub fn with_arity(name: IString, arity: Arity) -> FreeSymbol {
    let core = SymbolCore::with_arity(name, arity);
    FreeSymbol { core }
  }
}

impl Symbol for FreeSymbol {
  fn core(&self) -> &SymbolCore {
    &self.core
  }

  fn core_mut(&mut self) -> &mut SymbolCore {
    &mut self.core
  }
}


impl Formattable for FreeSymbol{
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    match style {

      FormatStyle::Debug => write!(f, "Symbol<{}>", self.core().name),

      _ => write!(f, "{}", self.core().name),

    }
  }
}

impl_display_debug_for_formattable!(FreeSymbol);