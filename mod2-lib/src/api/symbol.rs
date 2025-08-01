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

use crate::{
  api::{
    Arity,
    BxTerm,
    DagNode,
    DagNodePtr,
    TermPtr,
  },
  core::{
    format::{FormatStyle, Formattable},
    pre_equation::RulePtr,
    symbol::{
      SortTable,
      SymbolCore,
      OpDeclaration
    },
    rewriting_context::RewritingContext,
    sort::KindPtr,
    strategy::Strategy,
    EquationalTheory,
    SortIndex,
    SymbolIndex
  },
  impl_display_debug_for_formattable,
  HashType,
};


pub type SymbolPtr = UnsafePtr<dyn Symbol>;
pub type SymbolSet = Set<SymbolPtr>;

pub trait Symbol {
  // decl_as_any_ptr_fns!(Symbol);
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  fn as_ptr(&self) -> SymbolPtr;

  /// A type-erased way of asking a symbol to make a term of compatible type.
  fn make_term(&self, args: Vec<BxTerm>) -> BxTerm;

  /// A type-erased way of asking a symbol to make a DAG node of compatible type.
  fn make_dag_node(&self, args: *mut u8) -> DagNodePtr;

  // region Member Getters and Setters
  /// Trait level access to members for shared implementation
  fn core(&self) -> &SymbolCore;
  fn core_mut(&mut self) -> &mut SymbolCore;

  fn theory(&self) -> EquationalTheory;

  #[inline(always)]
  fn is_variable(&self) -> bool {
    self.core().is_variable()
  }

  #[inline(always)]
  fn name(&self) -> IString {
    self.core().name.clone()
  }

  /// Same as `get_order` or `get_hash_value`, used for "structural_hash".
  ///
  /// The semantics of a symbol are not included in the hash itself, as symbols are unique names by definition.
  #[inline(always)]
  fn hash(&self) -> HashType {
    self.core().hash()
  }

  #[inline(always)]
  fn arity(&self) -> Arity {
    self.sort_table().arity()
  }

  #[inline(always)]
  fn sort_table(&self) -> &SortTable {
    &self.core().sort_table
  }

  #[inline(always)]
  fn sort_table_mut(&mut self) -> &mut SortTable {
    &mut self.core_mut().sort_table
  }

  #[inline(always)]
  fn domain_kind(&self, idx: usize) -> KindPtr {
    self.sort_table().domain_component(idx)
  }

  #[inline(always)]
  fn range_kind(&self) -> KindPtr {
    self.sort_table().range_kind()
  }

  #[inline(always)]
  fn index_within_parent_module(&self) -> SymbolIndex {
    self.core().index_within_parent_module
  }

  #[inline(always)]
  fn strategy(&self) -> Option<&Strategy> {
    self.core().strategy.as_deref()
  }

  #[inline(always)]
  fn standard_strategy(&self) -> bool {
    // Uses the standard strategy if `self.strategy` is `None`.
    self.core().strategy.is_none()
  }

  #[inline(always)]
  fn eager_argument(&self, arg_count: usize) -> bool {
    if let Some(strategy) = self.core().strategy.as_deref() {
      strategy.eager.contains(arg_count)
    } else {
      false
    }
  }

  #[inline(always)]
  fn evaluated_argument(&self, arg_count: usize) -> bool {
    if let Some(strategy) = self.core().strategy.as_deref() {
      strategy.evaluated.contains(arg_count)
    } else {
      false
    }
  }

  #[inline(always)]
  fn sort_constraint_free(&self) -> bool {
    self.core().sort_constraint_table.sort_constraint_free()
  }

  #[inline(always)]
  fn rules(&self) -> &Vec<RulePtr> {
    &self.core().rules
  }

  // endregion Accessors

  #[inline(always)]
  fn compare(&self, other: &dyn Symbol) -> Ordering {
    self.hash().cmp(&other.hash())
  }

  // Compiler related methods

  #[inline(always)]
  fn add_op_declaration(&mut self, op_declaration: OpDeclaration) {
    self.core_mut().add_op_declaration(op_declaration);
  }

  /// Called from `Module::close_theory()`
  #[inline(always)]
  fn compile_op_declarations(&mut self) {
    let symbol_ptr = self.as_ptr();
    self.core_mut().sort_table.compile_op_declaration(symbol_ptr)
  }

  // Rewriting related methods

  /// Performs symbol-specific equational rewriting on the given DAG node.
  /// This virtual method is called by the reduction machinery to apply equations
  /// and built-in operations specific to this symbol type. Returns `true` if
  /// the subject was modified.
  fn rewrite(&mut self, subject: DagNodePtr, context: &mut RewritingContext) -> bool;


  fn fast_compute_true_sort(&mut self, mut subject: DagNodePtr, context: &mut RewritingContext) {
    // let root = self.root.unwrap();
    match self.core().unique_sort_index {
      SortIndex::SlowCaseUniqueSort => {
        // most general case
        self.slow_compute_true_sort(subject, context);
      }

      SortIndex::FastCaseUniqueSort => {
        // usual case
        subject.compute_base_sort();
      }

      other => {
        // unique sort case
        subject.set_sort_index(other);
      }
    }
  }

  /// Computes the true sort of root.
  fn slow_compute_true_sort(&mut self, subject: DagNodePtr, context: &mut RewritingContext) {
    self.core_mut()
        .sort_constraint_table
        .constrain_to_smaller_sort(subject.clone(), context);
  }

  fn constrain_to_smaller_sort(&mut self, subject: DagNodePtr, context: &mut RewritingContext) {
    self.core_mut().sort_constraint_table.constrain_to_smaller_sort(subject, context);
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
