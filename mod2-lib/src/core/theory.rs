/*!

We occasionally have a need to explicitly identify an equational theory, for example, when upgrading a `DagNodeCore`.

*/

use std::fmt::Write;
use crate::core::format::{FormatStyle, Formattable};
use crate::core::symbol::{SymbolAttributes, SymbolType};
use crate::impl_display_debug_for_formattable;

#[derive(Copy, Clone, Eq, PartialEq, Default, Hash)]
pub enum EquationalTheory {
  #[default]
  Free = 0,
  // ACU,
  // AU,
  // CUI,
  Variable,
  NA,
}

impl EquationalTheory {
  pub fn name_str(self) -> &'static str {
    match self {
      EquationalTheory::Free => "Free",
      EquationalTheory::Variable => "Variable",
      EquationalTheory::NA => "NA",
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
  if symbol_type.is_build_in_data_type() {
    EquationalTheory::NA
  } else if symbol_type == SymbolType::Variable {
    EquationalTheory::Variable
  } else { 
    EquationalTheory::Free
  }
}
