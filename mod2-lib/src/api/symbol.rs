/*!

Every theory's symbol type must implement the `Symbol` trait. The concrete symbol type should
have a `SymbolCore` member that can be accessed through the trait method `Symbol::core()`
and `Symbol::core_mut()`. This allows a lot of shared implementation in `SymbolCore`.

*/

use std::{
  any::Any,
  cmp::Ordering
};

use mod2_abs::{decl_as_any_ptr_fns, IString, Set, UnsafePtr};

use crate::{api::{Arity, term::BxTerm}, core::{
  format::{FormatStyle, Formattable},
  symbol::{
    SortTable,
    SymbolCore,
    OpDeclaration
  },
  sort::kind::KindPtr,
}, impl_display_debug_for_formattable, HashType};
use crate::core::strategy::Strategy;

pub type SymbolPtr = UnsafePtr<dyn Symbol>;
pub type SymbolSet = Set<SymbolPtr>;

pub trait Symbol {
  // decl_as_any_ptr_fns!(Symbol);
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  fn as_ptr(&self) -> SymbolPtr;
  
  /// A type-erased way of asking a symbol to make a term of compatible type.
  fn make_term(&self, args: Vec<BxTerm>) -> BxTerm;
  
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
  fn hash(&self) -> HashType {
    self.core().hash()
  }

  #[inline(always)]
  fn arity(&self) -> Arity {
    self.core().arity()
  }

  #[inline(always)]
  fn sort_table(&self) -> &Option<SortTable> {
    &self.core().sort_table
  }

  #[inline(always)]
  fn sort_table_mut(&mut self) -> &mut Option<SortTable> {
    &mut self.core_mut().sort_table
  }

  #[inline(always)]
  fn domain_kind(&self, idx: usize) -> KindPtr {
    let sort_table = self.sort_table().as_ref().expect("cannot fetch domain kind of symbol with no sort table");
    sort_table.domain_component(idx)
  }

  #[inline(always)]
  fn range_kind(&self) -> KindPtr {
    let sort_table = self.sort_table().as_ref().expect("cannot fetch range kind of symbol with no sort table");
    sort_table.range_kind()
  }
  
  #[inline(always)]
  fn strategy(&self) -> &Strategy {
    &self.core().strategy
  }
  
  // endregion Accessors

  #[inline(always)]
  fn compare(&self, other: &dyn Symbol) -> Ordering {
    self.hash().cmp(&other.hash())
  }

  #[inline(always)]
  fn add_op_declaration(&mut self, self_ptr: SymbolPtr, op_declaration: OpDeclaration) {
    self.core_mut().add_op_declaration(self_ptr, op_declaration);
  }
  
  /// Called from `Module::close_theory()`
  #[inline(always)]
  fn compile_op_declarations(&mut self) {
    if let  Some(sort_table) = self.core_mut().sort_table.as_mut() {
      sort_table.compile_op_declaration()
    }
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
