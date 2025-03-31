#![feature(assert_matches)]
#![allow(unused)]

mod parser;

pub use mod2_lib::core::Module;

#[cfg(test)]
mod tests;


// Global Configuration
/// Indentation amount for displayed structures.
pub(crate) const DISPLAY_INDENT: usize = 2;

