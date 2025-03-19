use mod2_abs::IString;
use crate::{
  core::{
    symbol_core::SymbolCore,
    format::{FormatStyle, Formattable}
  },
  api::{
    symbol::Symbol,
    Arity
  },
  impl_display_debug_for_formattable
};

pub struct FreeSymbol {
  core: SymbolCore
}

impl FreeSymbol {
  pub(crate) fn with_arity(name: IString, arity: Arity) -> FreeSymbol {
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