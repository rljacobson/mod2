#![feature(assert_matches)]
#![allow(unused)]

mod parser;
mod module;

#[cfg(test)]
mod tests;


// Global Configuration
/// Indentation amount for displayed structures.
pub(crate) const DISPLAY_INDENT: usize = 2;

