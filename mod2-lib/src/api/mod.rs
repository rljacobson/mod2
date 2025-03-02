#![allow(unused_imports, dead_code)]
/*!

The public API of the library.

*/

pub mod atom;
pub mod symbol;
mod variable;
pub(crate) mod term;
pub(crate) mod dag_node;
pub mod free_theory;

// Special Values
// ToDo: Do UNDEFINED the right way. Is this great? No. But it's convenient.
pub(crate) const UNDEFINED: i32 = -1;
const NONE:      i32 = -1;
const ROOT_OK:   i32 = -2;

// Small utility types used throughout
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Arity {
  Any,
  None,
  Unspecified,

  Variadic,
  Value(u16)
}

/*
impl From<Arity> for i16 {
  fn from(arity: Arity) -> Self {
    match arity {

      Arity::None
      | Arity::Unspecified => -2,

      Arity::Any
      | Arity::Variadic => -1,

      Arity::Value(val) => val

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
      return Arity::Value(i)
    }
  }
}
*/
