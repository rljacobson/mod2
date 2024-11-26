/*!

A symbol is a name that cannot be rebound (or unbound). It is considered a concrete value. A symbol exists only within
an equational theory. The `Symbol` struct contains the common implementation of all symbols and defines the API for
symbols. The `Symbol` struct delegates to a `TheorySymbol` for theory-specific implementation.

*/

use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::write;
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

/// Special arity values.
// ToDo: Make arity a newtype.
pub const VARIADIC   : i16 = -1;
pub const UNSPECIFIED: i16 = -2;

pub struct Symbol {
  /// `NamedEntity` members
  pub name       : IString,

  // Symbol members
  pub arity      : i16, // -1 means variadic, -2 means unspecified
  pub symbol_type: SymbolType,
  // ToDo: Should `sort_spec` be a member of `SymbolType`?
  pub sort_spec  : Option<BxSortSpec>,

  // Maude uses `indexWithinParent` "in preference to a raw pointer comparison so the
  // ordering does not depend on the vagaries of the memory allocator, which can mess up
  // test suite .out files." We use pointer comparison until evidence recommends otherwise.
  // index_within_parent: u32,

  /// Maude uses `orderHash` (hash of upper byte arity, lower bytes increasing monotonically) in a `compare` method.
  /// We can't use only the name, because two different modules might have distinct symbols of the same name.
  pub order_hash : u32,

  /// The theory-specific implementation of a symbol. (An alternative design is used for `PreEquation`, where the
  /// subtype is implemented as an enum.)
  pub theory_symbol: Option<Box<dyn TheorySymbol>>,
}

impl Symbol {

  /// Creates a new implicitly defined / generic symbol.
  pub fn new(name: IString) -> Symbol {
    Symbol{
      name,
      arity        : UNSPECIFIED,
      order_hash   : Symbol::new_order_hash(UNSPECIFIED),
      symbol_type  : SymbolType::default(),
      sort_spec    : None,
      theory_symbol: None,
    }
  }

  // ToDo: It would be better if we had a static object for constants like this.
  // ToDo: If a static object is heap allocated and then given to a module to own, its memory will be reclaimed along
  //       with the rest of the symbols for that module.

  /// Constructs a new heap-allocated symbol representing the
  /// "system" true constant, returning an owning mutable pointer.
  pub fn true_literal() -> SymbolPtr {
    heap_construct!(
      Symbol{
        name        : IString::from("true"),
        arity       : UNSPECIFIED,
        order_hash  : Symbol::new_order_hash(UNSPECIFIED),
        symbol_type : SymbolType{
          core_type : CoreSymbolType::SystemTrue,
          attributes: Default::default(),
        },
        sort_spec    : None,
        theory_symbol: None,
      }
    )
  }

  /// Constructs a new heap-allocated symbol representing the
  /// "system" false constant, returning an owning mutable pointer.
  pub fn false_literal() -> SymbolPtr {
    heap_construct!(
      Symbol{
        name        : IString::from("false"),
        arity       : UNSPECIFIED,
        order_hash  : Symbol::new_order_hash(UNSPECIFIED),
        symbol_type : SymbolType{
          core_type : CoreSymbolType::SystemFalse,
          attributes: Default::default(),
        },
        sort_spec    : None,
        theory_symbol: None,
      }
    )
  }

  pub fn new_order_hash(arity: i16) -> u32 {
    static mut SYMBOL_COUNT: u32 = 0;
    unsafe{
      SYMBOL_COUNT += 1;
      let arity: u32 = arity as u32;
      (arity << 24) | SYMBOL_COUNT
    }
  }

  pub fn compare_by_order_hash(&self, other: &Self) -> Ordering {
    self.order_hash.cmp(&other.order_hash)
  }

}

//  region Order and Equality impls
impl PartialOrd for Symbol {
  #[inline(always)]
  fn partial_cmp(&self, other: &Symbol) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Symbol {
  #[inline(always)]
  fn cmp(&self, other: &Symbol) -> Ordering {
    let self_address = std::ptr::addr_of!(self) as usize;
    let other_address = std::ptr::addr_of!(other) as usize;

    self_address.cmp(&other_address)
  }
}

impl Eq for Symbol {}

impl PartialEq for Symbol {
  #[inline(always)]
  fn eq(&self, other: &Symbol) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}
// endregion


/// Equational theory-specific implementations implement the `TheorySymbol` trait.
pub trait TheorySymbol {}


pub fn symbol_for_symbol_type(symbol_type: &SymbolType) -> Box<dyn TheorySymbol> {
  // Variable trumps all.
  if symbol_type.core_type == CoreSymbolType::Variable {
    Box::new(VariableSymbol::default())
  }
  else if symbol_type.attributes.contains(SymbolAttribute::Associative) {
    if symbol_type.attributes.contains(SymbolAttribute::Commutative) {
      // ACU Theory
      // unimplemented!("ACU Theory is not implemented")
      // Everything is a free symbol until we implement something else.
      Box::new(FreeSymbol::default())
    }
    else {
      // AU Theory
      // unimplemented!("AU Theory is not implemented")
      // Everything is a free symbol until we implement something else.
      Box::new(FreeSymbol::default())
    }
  }
  else if symbol_type.attributes.contains(SymbolAttribute::Commutative) {
    // CUI Theory
    // unimplemented!("CUI Theory is not implemented")
    // Everything is a free symbol until we implement something else.
    Box::new(FreeSymbol::default())
  }
  else {
    // Free Theory
    Box::new(FreeSymbol::default())
  }
}
