#![feature(ptr_as_ref_unchecked)]
#![feature(ptr_metadata)]
#![feature(vec_into_raw_parts)]
#![allow(dead_code)]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod api;
pub mod core;

// Global Configuration
/// Indentation amount for displayed structures.
pub(crate) const DISPLAY_INDENT: usize = 2;

/// The type to use for symbol, term, and DAG node hashes.
pub type HashType = u32;

// Sentinel Values
// ToDo: Do UNDEFINED the right way. Is this great? No. But it's convenient.
const UNDEFINED: i32 = -1;
const NONE     : i32 = -1;
const ROOT_OK  : i32 = -2;

pub fn add(left: u64, right: u64) -> u64 {
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
