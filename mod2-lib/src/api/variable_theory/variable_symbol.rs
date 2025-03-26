use mod2_abs::IString;
use crate::{
  api::{
    symbol::Symbol,
    Arity
  },
  core::{
    sort::SortPtr,
    symbol::{
      SymbolAttributes,
      SymbolType,
      SymbolCore
    },
    format::{FormatStyle, Formattable},
  },
  impl_display_debug_for_formattable,
};

pub struct VariableSymbol {
  core: SymbolCore
}

impl VariableSymbol {
  pub fn new(
    name       : IString,
    arity      : Arity,
    attributes : SymbolAttributes,
    symbol_type: SymbolType,
  ) -> VariableSymbol {
    let core = SymbolCore::new(name, arity, attributes, symbol_type);
    VariableSymbol{ core }
  }
  
  pub(crate) fn with_name(name: IString) -> VariableSymbol {
    let core = SymbolCore::with_name(name);
    VariableSymbol { core }
  }
}

impl Symbol for VariableSymbol {
  fn core(&self) -> &SymbolCore {
    &self.core
  }

  fn core_mut(&mut self) -> &mut SymbolCore {
    &mut self.core
  }
}


impl Formattable for VariableSymbol{
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    match style {

      FormatStyle::Debug => write!(f, "VariableSymbol<{}>", self.core().name),

      _ => write!(f, "{}", self.core().name),

    }
  }
}

impl_display_debug_for_formattable!(VariableSymbol);