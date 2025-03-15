/*!

Every theory's symbol type must implement the `Symbol` trait. The concrete symbol type should
have a `SymbolCore` member that can be accessed through the trait method `Symbol::core()`
and `Symbol::core_mut()`. This allows a lot of shared implementation in `SymbolCore`.

*/

use std::cmp::Ordering;

use mod2_abs::{IString, Set, UnsafePtr};

use crate::{
  api::Arity,
  core::{
    format::{FormatStyle, Formattable},
    symbol_core::SymbolCore,
    sort::sort_table::SortTable
  },
  impl_display_debug_for_formattable,
};


pub type SymbolPtr = UnsafePtr<dyn Symbol>;
pub type SymbolSet = Set<SymbolPtr>;

pub trait Symbol {
  // region Member Getters and Setters
  /// Trait level access to members for shared implementation
  fn core(&self) -> &SymbolCore;
  fn core_mut(&mut self) -> &mut SymbolCore;

  #[inline(always)]
  fn is_variable(&self) -> bool {
    self.core().is_variable()
  }

  #[inline(always)]
  fn name(&self) -> IString {
    self.core().name.clone()
  }

  /// Same as `get_order` or `get_hash_value`, used for "semantic hash".
  ///
  /// The semantics of a symbol are not included in the hash itself, as symbols are unique names by definition.
  #[inline(always)]
  fn hash(&self) -> u32 {
    self.core().hash()
  }

  #[inline(always)]
  fn arity(&self) -> Arity {
    self.core().arity
  }

  #[inline(always)]
  fn sort_constraint_table(&self) -> &Option<SortTable> {
    &self.core().sort_table
  }

  #[inline(always)]
  fn sort_constraint_table_mut(&mut self) -> &mut Option<SortTable> {
    &mut self.core_mut().sort_table
  }
  
  // endregion Accessors
  
  fn compare(&self, other: &dyn Symbol) -> Ordering {
    self.hash().cmp(&other.hash())
  }
}


impl Formattable for dyn Symbol{
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    match style {
      
      FormatStyle::Debug => write!(f, "Symbol<{}>", self.core().name),
      
      _ => write!(f, "{}", self.core().name),
      
    }
  }
}

impl_display_debug_for_formattable!(dyn Symbol);