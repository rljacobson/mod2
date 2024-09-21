/*!

Types/type aliases that abstract over the implementing backing type.

A motivating example is the `RcCell` type, a reference-counting smart pointer that provides run-time checked mutable
access to its contents and supports weak references. A number of external crates could provide this functionality. This
module redirects to whatever chosen implementation we want. (Ironically, `RcCell` is no longer widely used in this code
base.)

Most of this module consists of either pub imports, type aliases, or little snippets of type definitions.

*/

#![allow(unused)]
mod nat_set;
mod rccell;
mod heap;


// A fast hash set and hash map
pub use std::collections::{HashSet, HashMap};
use std::fmt::Display;
pub use tiny_logger::{log, set_verbosity, Channel};



// A set of (small) natural numbers
pub use nat_set::NatSet;



// Reference counted pointers with mutable stable, and complementary weak pointers.
pub use rccell::{rc_cell, RcCell, WeakCell};



// Heap construction/destruction
pub use heap::{heap_construct, heap_destroy};


use ustr::Ustr;
/// Interned strings. Create an interned string with `IString::from(..)`
pub type IString = Ustr;



// Numeric Types
/// Nonnegative Integers
pub type NaturalNumber = u64;
/// Signed Integers
pub type Integer       = i16;
/// Floating Point Numbers
pub type Float         = f64;

use std::iter::once;
use crate::add;

/**
Join an iterator of strings, which doesn't exist in the stdlib. (C.f. `Vec::join(…)`)

From: <https://stackoverflow.com/a/66951473>
Usage:

    let iter = [1, 3, 5, 7, 9].iter().cloned();
    println!("{:?}", join_iter(iter, |v| v - 1).collect::<Vec<_>>());
    // [1, 2, 3, 4, 5, 6, 7, 8, 9]

    let iter = ["Hello", "World"].iter().cloned();
    let sep = ", ";
    println!("{:?}", join_iter(iter, |_| sep).collect::<String>());
    // "Hello, World"
 */
pub fn join_iter<T>(mut iter: impl Iterator<Item = T>, sep: impl Fn(&T) -> T)
  -> impl Iterator<Item = T>
{
  iter
      .next()
      .into_iter()
      .chain(iter.flat_map(move |s| once(sep(&s)).chain(once(s))))
}

/// Join a list of things that can be displayed as string with a given separator.
///
/// This is a convenience function that defers to `join_iter`.
pub fn join_string<T:Display>(mut iter: impl Iterator<Item = T>, sep: &str) -> String {
  join_iter(iter.map(|t| t.to_string()), |_| sep.to_string()).collect::<String>()
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn join_iter_test() {
    let iter = [1, 3, 5, 7, 9].iter().cloned();
    let joined = format!("{:?}", join_iter(iter, |v| v - 1).collect::<Vec<_>>());
    assert_eq!(joined, "[1, 2, 3, 4, 5, 6, 7, 8, 9]");
    // [1, 2, 3, 4, 5, 6, 7, 8, 9]

    let iter = ["Hello", "World"].iter().cloned();
    let sep = ", ";
    let joined = format!("{:?}", join_iter(iter, |_| sep).collect::<String>());
    assert_eq!(joined, "\"Hello, World\"");
    // "Hello, World"
  }

  #[test]
  fn join_string_test(){
    let list = [1, 3, 5, 7, 9];
    let sep = ", ";
    let joined = join_string(list.iter(), sep);
    assert_eq!(joined, "1, 3, 5, 7, 9");
  }
}
