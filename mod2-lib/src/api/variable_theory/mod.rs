use std::fmt::{Display, Formatter};
use crate::api::symbol::{Symbol, SymbolPtr};

/// The `VariableType` of a variable determines what the variable is able to bind to. A `Blank` variable binds to a
/// single `Term`, a `Sequence` variable binds to a sequence of one or more `Term`s, and a `NullSequence` binds to a
/// sequence of zero or more `Term`s.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VariableType {
  Blank,          // Singleton wildcard (a blank)
  Sequence,       // One-or-more wildcard (a blank sequence)
  NullSequence,   // Zero-or-more wildcard (a blank null sequence)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Variable {
  pub symbol:        SymbolPtr,
  pub variable_type: VariableType,
}

impl Display for Variable {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let symbol: &Symbol = unsafe {
      &*(self.symbol)
    };

    match self.variable_type {
      VariableType::Blank        => write!(f, "{}_",   symbol),
      VariableType::Sequence     => write!(f, "{}__",  symbol),
      VariableType::NullSequence => write!(f, "{}___", symbol),
    }
  }
}
