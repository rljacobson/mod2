/*!

We occasionally have a need to explicitly identify an equational theory, for example, when upgrading a `DagNodeCore`.

*/

use std::fmt::Write;
use crate::core::format::{FormatStyle, Formattable};
use crate::core::symbol::{SymbolAttributes, SymbolType};
use crate::impl_display_debug_for_formattable;

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash)]
#[repr(u8)]
pub enum EquationalTheory {
  #[default]
  Free = 0,
  // ACU,
  // AU,
  // CUI,
  Variable,
  Bool,
  Float,
  Integer,
  NaturalNumber,
  String,
}

impl EquationalTheory {
  pub fn name_str(self) -> &'static str {
    match self {
      EquationalTheory::Free => "Free",
      EquationalTheory::Variable => "Variable",
      EquationalTheory::Bool => "Bool",
      EquationalTheory::Float => "Float",
      EquationalTheory::Integer => "Integer",
      EquationalTheory::NaturalNumber => "NaturalNumber",
      EquationalTheory::String => "String",
    }
  }
}

impl Formattable for EquationalTheory {
  fn repr(&self, out: &mut dyn Write, _style: FormatStyle) -> std::fmt::Result {
    write!(out, "{}Theory", self.name_str())
  }
}

impl_display_debug_for_formattable!(EquationalTheory);

pub fn theory_from_symbol_attributes(symbol_type: SymbolType, _attributes: SymbolAttributes) -> EquationalTheory {
  match symbol_type {
    SymbolType::Variable      => EquationalTheory::Variable,
    SymbolType::True          => EquationalTheory::Bool,
    SymbolType::False         => EquationalTheory::Bool,
    SymbolType::String        => EquationalTheory::String,
    SymbolType::Float         => EquationalTheory::Float,
    SymbolType::Integer       => EquationalTheory::Integer,
    SymbolType::NaturalNumber => EquationalTheory::NaturalNumber,
    _                         => EquationalTheory::Free
  }
}
