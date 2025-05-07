#![feature(assert_matches)]
#![allow(unused)]

mod parser;
#[cfg(test)]
mod tests;

pub use mod2_lib::core::Module;

pub use parser::{parse_to_module, parse_to_term};


// Global Configuration
/// Indentation amount for displayed structures.
pub(crate) const DISPLAY_INDENT: usize = 2;

