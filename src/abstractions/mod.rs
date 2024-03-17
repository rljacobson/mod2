#![allow(unused)]
mod nat_set;
mod rccell;



// A fast hash set and hash map
pub use std::collections::{HashSet, HashMap};



pub use tiny_logger::{log, set_verbosity, Channel};



// A set of (small) natural numbers
pub use nat_set::NatSet;



// Reference counted pointers with mutable stable, and complementary weak pointers.
pub use rccell::{rc_cell, RcCell, WeakCell};



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
/**
Join an iterator of strings, which doesn't exist in the stdlib. (C.f. `Vec::join(…)`)

From: https://stackoverflow.com/a/66951473
Usage:

    let iter = [1, 3, 5, 7, 9].iter().cloned();
    println!("{:?}", join_iter(iter, |v| v - 1).collect::<Vec<_>>());
    // [1, 2, 3, 4, 5, 6, 7, 8, 9]

    let iter = ["Hello", "World"].iter().cloned();
    let sep = ", ";
    println!("{:?}", join_iter(iter, |_| sep).collect::<String>());
    // "Hello, World"
 */
pub fn join_iter<T>(mut iter: impl Iterator<Item = T>, sep: impl Fn(&T) -> T) -> impl Iterator<Item = T> {
  let j: Integer = 374u64.try_into().unwrap_or(-1);
  iter
      .next()
      .into_iter()
      .chain(iter.flat_map(move |s| once(sep(&s)).chain(once(s))))
}
