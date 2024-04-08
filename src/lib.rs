#![allow(unused)]
#![doc(include = "../doc/DesignNodes.md")]
#![doc(include = "../doc/Syntax.md")]
#![doc(include = "../doc/QuickStart.md")]`

mod parser;
pub mod abstractions;
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
