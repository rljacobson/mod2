#![feature(ptr_as_ref_unchecked)]
#![feature(ptr_metadata)]
#![allow(dead_code)]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod api;
mod core;

// Configuration

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
