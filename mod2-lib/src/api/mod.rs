#![allow(unused_imports, dead_code)]
/*!

The public API of the library.

*/

pub mod symbol;
pub mod term;
pub(crate) mod dag_node;
pub mod variable_theory;
pub mod free_theory;
pub mod built_in;
mod dag_node_cache;

// Special Values
// ToDo: Do UNDEFINED the right way. Is this great? No. But it's convenient.
pub const UNDEFINED: i32 = -1;
pub const NONE:      i32 = -1;
pub const ROOT_OK:   i32 = -2;

// Small utility types used throughout
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Arity {
  Any,          // Synonym for variadic?
  None,         // Constant
  Unspecified,  // Missing value
  // ToDo: Variadic could be just positive arity together with assoc attribute.
  Variadic,
  Value(u16)    // Specific value
}

impl Arity {
  #[inline(always)]
  pub fn as_numeric(&self) -> u32 {
    if let Arity::Value(v) = self {
      *v as u32
    } else {
      0
    }
  }
}

impl From<Arity> for i16 {
  fn from(arity: Arity) -> Self {
    match arity {

      Arity::None
      | Arity::Unspecified => -2,

      Arity::Any
      | Arity::Variadic => -1,

      Arity::Value(val) => val as i16

    }
  }
}

impl From<i16> for Arity {
  fn from(i: i16) -> Self {
    if i < -2 {
      panic!("Negative arity encountered: {}", i);
    } else if i == -2 {
      return Arity::Unspecified;
    } else if i == -1 {
      return Arity::Variadic;
    } else {
      return Arity::Value(i as u16)
    }
  }
}
