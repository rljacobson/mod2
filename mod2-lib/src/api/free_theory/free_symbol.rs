use mod2_abs::{impl_as_any_ptr_fns, IString};
use crate::{
  core::{
    symbol::{
      SymbolAttributes,
      SymbolType,
      SymbolCore
    },
    sort::SortPtr,
    format::{FormatStyle, Formattable},
    dag_node_core::DagNodeCore,
    EquationalTheory
  },
  api::{
    symbol::{
      SymbolPtr,
      Symbol
    },
    Arity,
    free_theory::FreeTerm,
    term::{
      BxTerm,
      TermPtr
    },
    dag_node::DagNodePtr
  },
  impl_display_debug_for_formattable,
};
use crate::core::rewriting_context::RewritingContext;

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
  // impl_as_any_ptr_fns!(Symbol, FreeSymbol);
  fn as_any(&self) -> &dyn std::any::Any { self }
  fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
  fn as_ptr(&self) -> SymbolPtr {
    SymbolPtr::new(self as *const dyn Symbol as *mut dyn Symbol)
  }

  fn make_term(&self, args: Vec<BxTerm>) -> BxTerm {
    Box::new(FreeTerm::new(self.as_ptr(), args))
  }

  fn make_dag_node(&self, args: *mut u8) -> DagNodePtr {
    DagNodeCore::with_args(self.as_ptr(), args)
  }

  fn core(&self) -> &SymbolCore {
    &self.core
  }

  fn core_mut(&mut self) -> &mut SymbolCore {
    &mut self.core
  }

  fn theory(&self) -> EquationalTheory {
    EquationalTheory::Free
  }

  fn rewrite(&mut self, subject: DagNodePtr, context: &mut RewritingContext) -> bool {
    assert_eq!(self.as_ptr(), subject.symbol());
    if self.standard_strategy() {
      for mut arg in subject.iter_args() {
        arg.reduce(context);
      }
      // ToDo: Implement discrimination net
      todo!("free symbol rewriting");
      // self.discrimination_net.apply_replace(subject, context)
    } else {
      // ToDo: Implement nonstandard strategy rewriting
      unimplemented!("nonstandard strategy rewriting not implemented");
      // self.complex_strategy(subject, context)
    }
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

#[cfg(test)]
mod tests {
  use std::ops::Deref;
  use super::*;

  #[test]
  fn test_symbol_creation(){
    let f = FreeSymbol::with_arity("f".into(), Arity::new_unchecked(3));
    assert_eq!(f.arity().get(), 3);
    assert_eq!(f.name().deref(), "f");
  }
}
