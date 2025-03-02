#![feature(assert_matches)]
#![allow(unused)]
// #![doc(include = "../doc/DesignNodes.md")]
// #![doc(include = "../doc/Syntax.md")]
// #![doc(include = "../doc/QuickStart.md")]

mod parser;
mod theory;
mod core;
mod builtin;

// Global Configuration
/// Indentation amount for displayed structures.
pub(crate) const DISPLAY_INDENT: usize = 2;

// Numeric Types
/// Nonnegative Integers
pub type NaturalNumber = u64;
/// Signed Integers
pub type Integer       = i16;
/// Floating Point Numbers
pub type Float         = f64;


pub fn add(left: usize, right: usize) -> usize {
  left + right
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let result = add(2, 2);
    assert_eq!(result, 4);
  }
}
