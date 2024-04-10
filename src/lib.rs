#![allow(unused)]
#![doc = include_str!("../doc/QuickStart.md")]
#![doc = include_str!("../doc/DesignNotes.md")]
#![doc = include_str!("../doc/Syntax.md")]

mod parser;
mod abstractions;
mod theory;
mod core;
mod builtin;

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
