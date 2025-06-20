use mod2_abs::{impl_as_any_ptr_fns, IString};
use crate::{
  api::{
    symbol::{Symbol, SymbolPtr},
    Arity,
    term::{BxTerm, TermPtr},
    variable_theory::{VariableTerm, VariableDagNode},
    dag_node::DagNodePtr,
  },
  core::{
    sort::SortPtr,
    symbol::{
      SymbolAttributes,
      SymbolType,
      SymbolCore,
      OpDeclaration
    },
    format::{FormatStyle, Formattable},
    term_core::TermCore,
    EquationalTheory,
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

  pub fn sort(&self) -> SortPtr {
    // Maude: Temporary hack until sorts mechanism revised.
    let s = self.core().sort_table.get_op_declarations();
    assert_eq!(s.len(), 1usize, "s.length() != 1");
    let v: &OpDeclaration = s.first().unwrap();
    assert_eq!(v.len(), 1usize, "v.length() != 1");

    v[0]
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
    // Box::new(VariableTerm::new(self.name(), self.as_ptr()))
    panic!("cannot call Symbol::make_term() on a VariableSymbol");
  }

  #[inline(always)]
  fn core(&self) -> &SymbolCore {
    &self.core
  }

  #[inline(always)]
  fn core_mut(&mut self) -> &mut SymbolCore {
    &mut self.core
  }
  
  fn make_dag_node(&self, _: *mut u8) -> DagNodePtr {
    panic!("cannot call Symbol::make_dag_node() on a VariableSymbol");
  }
  
  fn theory(&self) -> EquationalTheory { EquationalTheory::Variable }
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