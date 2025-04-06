use mod2_abs::{impl_as_any_ptr_fns, IString};
use crate::{
  api::{
    symbol::{
      Symbol,
      SymbolPtr
    },
    Arity,
    term::BxTerm,
    variable_theory::VariableTerm
  },
  core::{
    sort::SortPtr,
    symbol::{
      SymbolAttributes,
      SymbolType,
      SymbolCore
    },
    format::{FormatStyle, Formattable},
    term_core::TermCore
  },
  impl_display_debug_for_formattable,
};

pub struct VariableSymbol {
  core: SymbolCore
}

impl VariableSymbol {
  #[inline(always)]
  pub fn new(
    name       : IString,
    arity      : Arity,
    attributes : SymbolAttributes,
    symbol_type: SymbolType,
  ) -> VariableSymbol {
    let core = SymbolCore::new(name, arity, attributes, symbol_type);
    VariableSymbol{ core }
  }

  #[inline(always)]
  pub(crate) fn with_name(name: IString) -> VariableSymbol {
    let core = SymbolCore::with_name(name);
    VariableSymbol { core }
  }
}

impl Symbol for VariableSymbol {
  // impl_as_any_ptr_fns!(Symbol, VariableSymbol);
  #[inline(always)]
  fn as_any(&self) -> &dyn std::any::Any { self }
  #[inline(always)]
  fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
  #[inline(always)]
  fn as_ptr(&self) -> SymbolPtr {
    SymbolPtr::new(self as *const dyn Symbol as *mut dyn Symbol)
  }

  fn make_term(&self, _args: Vec<BxTerm>) -> BxTerm {
    // Box::new(VariableTerm::new(name, self.as_ptr()))
    // Use `VariableTerm::new(name, self.as_ptr())`
    unimplemented!();
  }

  #[inline(always)]
  fn core(&self) -> &SymbolCore {
    &self.core
  }

  #[inline(always)]
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