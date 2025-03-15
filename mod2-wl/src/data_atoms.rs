/*!

Some examples of data atoms.

Notes:
 - `f64` does not implement `Hash` or `Eq`, so we use the `TotalF64` type from the `total_float_wrap` crate instead.
 - By convention, types implementing `DataAtom` should also implement a `new_atom(data: isize) -> Atom` method that creates a new `Atom::Data` containing a boxed `DataAtom` wrapping `data`, that is, a `Atom::Data(Box::new(Self(data)))`.
 - For unknown reasons, you need `use paste::paste;` when you use `implement_data_atom!`. Seems like a violation of macro hygiene.

ToDo: Why do uses of `implement_data_atom!` require `use paste::paste`?
*/

use std::{
  any::Any,
  fmt::Display
};
use once_cell::sync::Lazy;
use total_float_wrap::TotalF64;
use mod2_abs::IString;
use mod2_lib::{
  api::{
    Arity,
    atom::{
      Atom,
      DataAtom,
      implement_data_atom
    },
    symbol_core::{
      Symbol,
      SymbolPtr,
      SymbolAttribute,
      SymbolType
    }
  }
};

/// A machine-sized float implemented "manually".
///
/// The `implement_data_atom!` would normally be used instead, but `f64` doesn't implement `Hash` or `Eq`.
/// Instead, we implement `DataAtom` "manually". We use the `TotalF64` type from the `total_float_wrap`
/// crate instead since TotalF64 implements `Any + PartialEq + Eq + Hash`.
#[derive(PartialEq, Eq, Debug, Hash)]
pub struct FloatAtom(TotalF64);
impl FloatAtom {
  /// Creates a new `Atom::Data` containing a boxed `DataAtom` wrapping `data`
  pub fn new_atom(data: f64) -> Atom {
    Atom::Data(Box::new(FloatAtom(TotalF64::from(data))))
  }
}

impl Display for FloatAtom {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0.0)
  }
}

impl DataAtom for FloatAtom {

  fn as_any(&self) -> &dyn Any {
    self
  }

  fn eq(&self, other: &dyn DataAtom) -> bool {
    if let Some(other) = other.as_any().downcast_ref::<FloatAtom>() {
      self.0 == other.0
    } else {
      false
    }
  }

  fn symbol(&self) -> SymbolPtr {
    let ptr: *const Symbol = &*FLOAT_SYMBOL;
    ptr as SymbolPtr
  }
}

#[allow(non_upper_case_globals)]
pub static FLOAT_SYMBOL: Lazy<Symbol> = Lazy::new(|| {
  Symbol{
    name:        IString::from("Float"),  // The convention is `Name` is the symbol for `NameAtom`.
    arity:       Arity::Unspecified,                  // Data is generally "nullary", but we can leave it unspecified.
    attributes:  SymbolAttribute::Constructor.into(), // All `DataAtom`s have `SymbolAttribute::Constructor`.
    symbol_type: SymbolType::Data,                    // All `DataAtom`s have `symbol_type` `SymbolType::Data`.
    hash_value:  0,
  }
});

// For types that implement `Display + Any + PartialEq + Eq + Hash`, we can use the `implement_data_atom` macro.

// A string type
implement_data_atom!(String, String);
// A byte type
implement_data_atom!(Byte, u8);
// An integer type
implement_data_atom!(Integer, isize);
