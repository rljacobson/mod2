/*!

Every theory's term type must implement the `Term` trait. The concrete term type should
have a `TermCore` member that can be accessed through the trait method `Term::core()`
and `Term::core_mut()`. This allows a lot of shared implementation in `TermCore`.

Note that an implementer of `Term` must also implement `Formattable`. We do it this way rather than a blanket
implementation, because some terms have their own particular representation.

*/

use std::{
  cmp::Ordering,
  collections::HashMap,
  fmt::Display,
  hash::{Hash, Hasher},
  ops::Deref
};

use mod2_abs::{decl_as_any_ptr_fns, NatSet, RcCell, UnsafePtr};

use crate::{api::{
  dag_node::{DagNode, DagNodePtr},
  symbol::{
    Symbol,
    SymbolPtr,
    SymbolSet,
  },
  UNDEFINED,
  Arity
}, impl_display_debug_for_formattable, core::{
  sort::kind::KindPtr,
  format::Formattable,
  substitution::Substitution,
  term_core::{
    cache_node_for_term,
    clear_cache_and_set_sort_info,
    lookup_node_for_term,
    TermCore,
  },
  VariableInfo
}, HashType};
use crate::core::term_core::TermAttribute;
use crate::core::TermBag;

pub type BxTerm    = Box<dyn Term>;
pub type MaybeTerm = Option<TermPtr>;
pub type TermPtr   = UnsafePtr<dyn Term>;
pub type TermSet   = HashMap<u32, usize>;

pub trait Term: Formattable {
  // decl_as_any_ptr_fns!(Term);
  fn as_any(&self) -> &dyn std::any::Any;
  fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
  /// This is very unsafe. Make sure this term is heap allocated and pinned before calling.
  fn as_ptr(&self) -> TermPtr;
  fn copy(&self) -> BxTerm;
  
  fn hash(&self) -> HashType { self.core().hash_value }

  /// Returns a pointer to the normalized version of self. If a new term was created during
  /// normalization, it is returned. We also need to know if any subterm changed, so we also
  /// return a bool, and unless the term is the expression's top-most term, we will always need
  /// the new hash value, too. The returned tuple is thus `( Option<TermBx>, changed, new_hash)`.
  ///
  /// Note: The hash value of a term is first set in this method.
  fn normalize(&mut self, full: bool) -> (Option<BxTerm>, bool, HashType);

  fn core(&self)         -> &TermCore;
  fn core_mut(&mut self) -> &mut TermCore;

  // region Accessors

  /// Is the term stable?
  #[inline(always)]
  fn is_stable(&self) -> bool {
    self.core().is_stable()
  }

  /// A subterm "honors ground out match" if its matching algorithm guarantees never to return a matching subproblem
  /// when all the terms variables are already bound.
  #[inline(always)]
  fn honors_ground_out_match(&self) -> bool {
    self.core().honors_ground_out_match()
  }

  #[inline(always)]
  fn set_honors_ground_out_match(&mut self, value: bool) {
    self.core_mut().set_honors_ground_out_match(value)
  }

  #[inline(always)]
  fn is_eager_context(&self) -> bool {
    self.core().is_eager_context()
  }

  #[inline(always)]
  fn is_variable(&self) -> bool {
    self.core().is_variable()
  }

  #[inline(always)]
  fn ground(&self) -> bool {
    self.core().ground()
  }

  /// The handles (indices) for the variable terms that occur in this term or its descendants
  #[inline(always)]
  fn occurs_below(&self) -> &NatSet {
    self.core().occurs_below()
  }

  #[inline(always)]
  fn occurs_below_mut(&mut self) -> &mut NatSet {
    self.core_mut().occurs_below_mut()
  }

  #[inline(always)]
  fn occurs_in_context(&self) -> &NatSet {
    self.core().occurs_in_context()
  }

  #[inline(always)]
  fn occurs_in_context_mut(&mut self) -> &mut NatSet {
    self.core_mut().occurs_in_context_mut()
  }

  #[inline(always)]
  fn collapse_symbols(&self) -> &SymbolSet {
    self.core().collapse_symbols()
  }

  /// Returns an iterator over the arguments of the term
  fn iter_args(&self) -> Box<dyn Iterator<Item = TermPtr> + '_>;
  // Implement an empty iterator with:
  //    Box::new(std::iter::empty::<TermPtr>())

  #[inline(always)]
  fn symbol(&self) -> SymbolPtr {
    self.core().symbol()
  }

  /// Compute the number of nodes in the term tree
  fn compute_size(&self) -> i32 {
    let cached_size: i32 = self.core().cached_size.get();

    if cached_size != UNDEFINED {
      cached_size
    }
    else {
      let mut size = 1; // Count self.
      for arg in self.iter_args() {
        size += arg.compute_size();
      }

      self.core().cached_size.set(size);
      size
    }
  }

  fn kind(&self) -> Option<KindPtr> {
    self.core().kind
  }
  
  fn set_attribute(&mut self, attribute: TermAttribute) {
    self.core_mut().attributes.insert(attribute);
  }
  
  fn reset_attribute(&mut self, attribute: TermAttribute) {
    self.core_mut().attributes.remove(attribute);
  }

  // endregion Accessors


  // region Comparison Functions

  fn compare_term_arguments(&self, _other: &dyn Term) -> Ordering;
  fn compare_dag_arguments(&self, _other: &dyn DagNode) -> Ordering;

  #[inline(always)]
  fn compare_dag_node(&self, other: &dyn DagNode) -> Ordering {
    if self.symbol().hash() == other.symbol().hash() {
      self.compare_dag_arguments(other)
    } else {
      // We only get equality, not ordering, from comparing hashes, so when hashes are unequal, we defer to compare.
      self.symbol().compare(other.symbol().deref())
    }
  }

  fn partial_compare(&self, partial_substitution: &mut Substitution, other: &dyn DagNode) -> Option<Ordering> {
    if !self.is_stable() {
      // Only used for `VariableTerm`
      return self.partial_compare_unstable(partial_substitution, other);
    }

    if std::ptr::addr_eq(self.symbol().as_ptr(), other.symbol().as_ptr()) {
      // Only used for `FreeTerm`
      return self.partial_compare_arguments(partial_substitution, other);
    }

    if self.symbol().compare(other.symbol().deref()) == Ordering::Less {
      Some(Ordering::Less)
    } else {
      Some(Ordering::Greater)
    }
  }

  #[inline(always)]
  fn compare(&self, other: &dyn Term) -> Ordering {
    let r = self.symbol().compare(other.symbol().deref());
    if r == Ordering::Equal {
      return self.compare_term_arguments(other);
    }
    r
  }

  /// Overridden in `VariableTerm`
  fn partial_compare_unstable(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> Option<Ordering> {
    None
  }

  /// Overridden in `FreeTerm`
  fn partial_compare_arguments(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> Option<Ordering> {
    None
  }

  // endregion Comparison Functions


  // region DAG Creation

  #[inline(always)]
  fn term_to_dag(&self, set_sort_info: bool) -> DagNodePtr {
    clear_cache_and_set_sort_info(set_sort_info);
    self.dagify()
  }

  /// Create a directed acyclic graph from this term. This trait-level implemented function takes care of structural
  /// sharing. Each implementing type will supply its own implementation of `dagify_aux(…)`, which recursively
  /// calls `dagify(…)` on its children and then converts itself to a type implementing DagNode, returning `DagNodePtr`.
  fn dagify(&self) -> DagNodePtr {
    let semantic_hash = self.hash();
    if let Some(dag_node) = lookup_node_for_term(semantic_hash) {
      return dag_node;
    }

    let dag_node = self.dagify_aux();
    cache_node_for_term(semantic_hash, dag_node);

    dag_node
  }

  /// Create a directed acyclic graph from this term. This method has the implementation-specific stuff.
  fn dagify_aux(&self) -> DagNodePtr;

  // endregion DAG Creation

  // region Compiler-related Function

/*
  /// Compiles the LHS automaton, returning the tuple `(lhs_automaton, subproblem_likely): (RcLHSAutomaton, bool)`
  fn compile_lhs(
    &self,
    match_at_top: bool,
    variable_info: &VariableInfo,
    bound_uniquely: &mut NatSet,
  ) -> (RcLHSAutomaton, bool);

  /// The theory-dependent part of `compile_rhs` called by `term_compiler::compile_rhs(…)`. Returns
  /// the `save_index`.
  fn compile_rhs_aux(
    &mut self,
    builder: &mut RHSBuilder,
    variable_info: &VariableInfo,
    available_terms: &mut TermBag,
    eager_context: bool,
  ) -> i32;
*/

  // A subterm "honors ground out match" if its matching algorithm guarantees never to return a matching subproblem
  // when all the terms variables are already bound.
  fn will_ground_out_match(&self, bound_uniquely: &NatSet) -> bool {
    self.honors_ground_out_match() && bound_uniquely.is_superset(&self.core().occurs_set)
  }

  fn analyse_constraint_propagation(&mut self, bound_uniquely: &mut NatSet);

  fn analyse_collapses(&mut self) {
    for mut arg in &mut self.iter_args() {
      arg.analyse_collapses();
    }

    if !self.is_variable() && self.collapse_symbols().is_empty() {
      self.set_attribute(TermAttribute::Stable);
    }
  }
  
  // There are no arguments to descend into for `VariableTerm`, so this is a no-op.
  // fn find_available_terms(&self, _available_terms: &mut TermBag, _eager_context: bool, _at_top: bool);
  
  /// For each argument term of this term, computes and updates the set of variables that
  /// occur in the context of the term and its subterms. The "context" of a term refers
  /// to the rest of the term in which it occurs (its parent term and sibling subterms).
  fn determine_context_variables(&self) {
    for mut term in self.iter_args() {
      // Insert parent's context set
      term.occurs_in_context_mut().union_in_place(self.occurs_in_context());

      for other_term in self.iter_args() {
        if *other_term != *term {
          // Insert sibling's occurs set
          term.occurs_in_context_mut().union_in_place(other_term.occurs_below());
        }
      }

      term.determine_context_variables();
    }
  }

  fn insert_abstraction_variables(&mut self, variable_info: &mut VariableInfo) {
    // Honors ground out match.
    let mut hgom = true;
    
    for mut term in self.iter_args() {
      term.insert_abstraction_variables(variable_info);
      hgom &= term.honors_ground_out_match();
    }
    
    self.set_honors_ground_out_match(hgom);
  }

  /// This method populates the sort information for the term and its subterms based on their
  /// symbol's sort declarations, validating them against the symbol's expected input and output
  /// types (domain and range components). (This is a method on `Symbol` in Maude.)
  fn fill_in_sort_info(&mut self) {
    let symbol = self.symbol();
    let sort_table = symbol.deref().sort_table().as_ref();
    assert!(sort_table.is_some(), "couldn't get component");
    let sort_table = sort_table.unwrap();
    let kind = sort_table.range_kind(); // should be const

    if symbol.arity() == Arity::Value(0) {
      self.set_sort_info(kind.clone(), sort_table.traverse(0, 0)); // HACK
      return;
    }

    let mut step = 0;
    let mut seen_args_count = 0;

    for mut term in self.iter_args() {
      term.fill_in_sort_info();
      // ToDo: Restore this assert.
      debug_assert_eq!(
        term.kind().unwrap(),
        symbol.domain_kind(seen_args_count),
        "component error on arg {} while computing sort of {}",
        seen_args_count,
        self.symbol()
      );
      step = sort_table.traverse(step as usize, term.core().sort_index as usize);
      seen_args_count += 1;
    }

    // ToDo: Restore this assert.
    // debug_assert_eq!(seen_args_count, seen_args_count, "bad # of args for op");
    self.set_sort_info(kind, step);
  }

  /// Sets the kind and the sort index of this term.
  fn set_sort_info(&mut self, kind: KindPtr, sort_index: i32) {
    let core = self.core_mut();
    core.kind = Some(kind);
    core.sort_index = sort_index;
  }

  /// Recursively collects the terms in a set for structural sharing.
  fn find_available_terms(&self, available_terms: &mut TermBag, eager_context: bool, at_top: bool);

  // endregion Compiler-related Function

}


// region trait impls for Term

// ToDo: Revisit whether `semantic_hash` is appropriate for the `Hash` trait.
// Use the `Term::compute_hash(…)` hash for `HashSet`s and friends.
impl Hash for dyn Term {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u32(self.hash())
  }
}

impl PartialEq for dyn Term {
  fn eq(&self, other: &Self) -> bool {
    self.hash() == other.hash()
  }
}

impl Eq for dyn Term {}

impl_display_debug_for_formattable!(dyn Term);
// endregion
