/*!

A symbol is a name that cannot be rebound (or unbound). It is considered a concrete value. A symbol exists only within
an equational theory. The `Symbol` struct contains the common implementation of all symbols and defines the API for
symbols. The `Symbol` struct delegates to a `TheorySymbol` for theory-specific implementation.

*/

use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::{abstractions::{
  IString,
  RcCell
}, heap_construct, rc_cell, theory::{
  free_theory::free_symbol::FreeSymbol,
  symbol_type::{
    CoreSymbolType,
    SymbolAttribute
    ,
    SymbolType
  },
  variable_theory::variable_symbol::VariableSymbol
}};
use crate::core::sort::sort_spec::BxSortSpec;

pub type SymbolPtr = *mut Symbol;

// Special arity values.
// ToDo: Make this a newtype.
pub const VARIADIC   : i16 = -1;
pub const UNSPECIFIED: i16 = -2;

pub struct Symbol {
  /// `NamedEntity` members
  pub name       : IString,
  pub arity      : i16, // -1 means variadic, -2 means unspecified
  pub symbol_type: SymbolType,
  // ToDo: Should `sort_spec` be a member of `SymbolType`?
  pub sort_spec  : Option<BxSortSpec>,

  // The theory-specific implementation of a symbol. (An alternative design is used for `PreEquation`, where the
  // subtype is implemented as an enum.)
  pub theory_symbol: Option<Box<dyn TheorySymbol>>,
}

impl Symbol {

  /// Creates a new implicitly defined / generic symbol.
  pub fn new(name: IString) -> Symbol {
    Symbol{
      name,
      arity        : UNSPECIFIED,
      symbol_type  : SymbolType::default(),
      sort_spec    : None,
      theory_symbol: None,
    }
  }

  // ToDo: It would be better if we had a static object for constants like this.

  /// Constructs a new heap-allocated symbol representing the
  /// "system" true constant, returning an owning mutable pointer.
  pub fn true_literal() -> SymbolPtr {
    let true_symbol: SymbolPtr = heap_construct!(
      Symbol{
          name        : IString::from("true"),
          arity       : UNSPECIFIED,
          symbol_type : SymbolType{
            core_type : CoreSymbolType::SystemTrue,
            attributes: Default::default(),
          },
          sort_spec    : None,
          theory_symbol: None,
    });

    true_symbol
  }

  /// Constructs a new heap-allocated symbol representing the
  /// "system" false constant, returning an owning mutable pointer.
  pub fn false_literal() -> SymbolPtr {
    let false_symbol: SymbolPtr = heap_construct!(Symbol{
      name        : IString::from("false"),
          arity       : UNSPECIFIED,
          symbol_type : SymbolType{
            core_type : CoreSymbolType::SystemFalse,
            attributes: Default::default(),
          },
          sort_spec    : None,
          theory_symbol: None,
    });

    false_symbol
  }

}

//  region Order and Equality impls
impl PartialOrd for Symbol {
  #[inline(always)]
  fn partial_cmp(&self, other: &Symbol) -> Option<Ordering> {
    let result = self.name.cmp(&other.name);
    Some(result)
  }
}

impl Ord for Symbol {
  #[inline(always)]
  fn cmp(&self, other: &Symbol) -> Ordering {
    self.name.cmp(&other.name)
  }
}

impl Eq for Symbol {}

impl PartialEq for Symbol {
  #[inline(always)]
  fn eq(&self, other: &Symbol) -> bool {
    self.name == other.name
  }
}
// endregion


/// Equational theory-specific implementations implement the `TheorySymbol` trait.
pub trait TheorySymbol {

}


pub fn symbol_for_symbol_type(symbol_type: &SymbolType) -> Box<dyn TheorySymbol> {
  // Variable trumps all.
  if symbol_type.core_type == CoreSymbolType::Variable {
    Box::new(VariableSymbol::default())
  }
  else if symbol_type.attributes.contains(SymbolAttribute::Associative) {
    if symbol_type.attributes.contains(SymbolAttribute::Commutative) {
      // ACU Theory
      unimplemented!("ACU Theory is not implemented")
    }
    else {
      // AU Theory
      unimplemented!("AU Theory is not implemented")
    }
  }
  else if symbol_type.attributes.contains(SymbolAttribute::Commutative) {
    // CUI Theory
    unimplemented!("CUI Theory is not implemented")
  }
  else {
    // Free Theory
    Box::new(FreeSymbol::default())
  }
}

