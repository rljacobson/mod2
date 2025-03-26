mod variable_symbol;
mod variable_term;

pub use variable_term::*;
pub use variable_symbol::*;

/// The `VariableType` of a variable determines what the variable is able to bind to. A `Blank` variable binds to a
/// single `Term`, a `Sequence` variable binds to a sequence of one or more `Term`s, and a `NullSequence` binds to a
/// sequence of zero or more `Term`s.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VariableType {
  Blank,          // Singleton wildcard (a blank)
  Sequence,       // One-or-more wildcard (a blank sequence)
  NullSequence,   // Zero-or-more wildcard (a blank null sequence)
}

