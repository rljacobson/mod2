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
  ops::{
    Deref,
    DerefMut
  }
};
use mod2_abs::{
  optimizable_int::OptU32
  ,
  NatSet,
  UnsafePtr
};
use crate::{
  api::{
    automaton::BxLHSAutomaton,
    dag_node::{
      DagNode,
      DagNodePtr
    },
    dag_node_cache::DagNodeCache,
    symbol::{
      Symbol,
      SymbolPtr,
      SymbolSet
    },
    variable_theory::VariableTerm
  },
  core::{
    automata::{
      BindingLHSAutomaton,
      CopyRHSAutomaton,
      RHSBuilder,
      TrivialRHSAutomaton
    },
    format::Formattable,
    sort::{
      KindPtr,
      SortIndex,
      SortPtr
    },
    substitution::Substitution,
    term_core::{
      TermAttribute,
      TermCore
    },
    TermBag,
    VariableInfo,
    VariableIndex
  },
  impl_display_debug_for_formattable,
  HashType,
};


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
  // fn copy(&self) -> TermPtr;

  /// Returns the structural hash computed in `Term::normalize()`
  fn structural_hash(&self) -> HashType { self.core().hash_value }

  /// Normalizes self. If a new (boxed) term was created during normalization, it is
  /// returned. We also need to know if any subterm changed, so we also return a bool,
  /// and unless the term is the expression's top-most term, we will always need the new
  /// hash value, too. The returned tuple is thus `(Option<TermBx>, changed, new_hash)`.

  /// Note: The structural hash value of a term is first set in this method. The algorithm used
  ///       to compute this hash should be kept in sync with the corresponding implementation
  ///       of `DagNode::structural_hash()`.
  fn normalize(&mut self, full: bool) -> (Option<BxTerm>, bool, HashType);

  /// Creates a deep copy of `self`.
  fn deep_copy(&self) -> BxTerm {
    // ToDo: Implement `SymbolTranslationMap`.
    let term = self.deep_copy_aux();
    // ToDo: Figure out what we want to do with line numbers
    // term.core_mut().line_number = self.core().line_number;
    term
  }

  /// The theory specific part of `deep_copy()`.
  fn deep_copy_aux(&self) -> BxTerm;

  /// This method is only used for `NATerm<T>`, but it is convenient to have it here.
  /// Overwrites `old_node` in place with a new `NADagNode<T>` with the same value as `self`.
  ///
  /// This method is on `Term` instead of `DagNode` because it is used in a context in which we only
  /// have a term available.
  fn overwrite_with_dag_node(&mut self, _old_node: DagNodePtr) -> DagNodePtr {
    panic!("Term::overwrite_with_dag_node() is not implemented for {}", std::any::type_name::<Self>());
  }


  // region Accessors

  fn core(&self)         -> &TermCore;
  fn core_mut(&mut self) -> &mut TermCore;

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
  fn compute_size(&self) -> u32 {
    // let cached_size: i32 = self.core().cached_size.get();
    if let Some(cached_size) = self.core().cached_size.get() {
      cached_size.get()
    }
    else {
      let mut size = 1; // Count self.
      for arg in self.iter_args() {
        size += arg.compute_size();
      }

      self.core().cached_size.set(Some(OptU32::new_unchecked(size)));
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

  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering;
  fn compare_dag_arguments(&self, other: DagNodePtr) -> Ordering;

  /// For equality check (`Term::equal_dag_node()`), just use `Term::compare_dag_node().is_equal()`.
  #[inline(always)]
  fn compare_dag_node(&self, other: DagNodePtr) -> Ordering {
    // Symbol equality has the semantics of pointer equality.
    if self.symbol() == other.symbol() {
      self.compare_dag_arguments(other)
    } else {
      // We only get equality, not ordering, from comparing hashes, so when hashes are unequal, we defer to compare.
      self.symbol().compare(other.symbol().deref())
    }
  }

  fn partial_compare(&self, partial_substitution: &mut Substitution, other: DagNodePtr) -> Option<Ordering> {
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
  fn partial_compare_unstable(&self, _partial_substitution: &mut Substitution, _other: DagNodePtr) -> Option<Ordering> {
    None
  }

  /// Overridden in `FreeTerm`
  fn partial_compare_arguments(&self, _partial_substitution: &mut Substitution, _other: DagNodePtr) -> Option<Ordering> {
    None
  }

  // endregion Comparison Functions


  // region DAG Creation

  #[inline(always)]
  fn term_to_dag(&self, set_sort_info: bool) -> DagNodePtr {
    let mut node_cache = DagNodeCache::new(set_sort_info);
    self.dagify(&mut node_cache)
  }

  /// Create a directed acyclic graph from this term. This trait-level implemented function takes care of structural
  /// sharing. Each implementing type will supply its own implementation of `dagify_aux(…)`, which recursively
  /// calls `dagify(…)` on its children and then converts itself to a type implementing DagNode, returning `DagNodePtr`.
  ///
  /// Terms should be normalized before dagification.
  #[allow(private_interfaces)]
  fn dagify(&self, node_cache: &mut DagNodeCache) -> DagNodePtr {
    let hash = self.structural_hash();
    if let Some(dag_node) = node_cache.get(hash) {
      return dag_node;
    }

    // The theory-specific part of `dagify`
    let mut dag_node = self.dagify_aux(node_cache);

    if node_cache.set_sort_info {
      assert!(self.core().sort_index.is_none(), "missing sort info");
      dag_node.set_sort_index(self.core().sort_index);
      dag_node.set_reduced();
    }

    node_cache.insert(hash, dag_node);
    dag_node
  }

  /// Create a directed acyclic graph from this term. This method has the implementation-specific stuff.
  #[allow(private_interfaces)]
  fn dagify_aux(&self, node_cache: &mut DagNodeCache) -> DagNodePtr;

  // endregion DAG Creation

  // region Compiler-related Function

  /// Compiles the LHS automaton, returning the tuple `(lhs_automaton, subproblem_likely): (RcLHSAutomaton, bool)`
  fn compile_lhs(
    &mut self,
    match_at_top  : bool,
    variable_info : &VariableInfo,
    bound_uniquely: &mut NatSet,
  ) -> (BxLHSAutomaton, bool) {
    // The theory-specific compilation occurs in `compile_aux()`.
    let (mut automaton, subproblem_likely) = self.compile_lhs_aux(match_at_top, variable_info, bound_uniquely);

    if let Some(save_index) = self.core_mut().save_index {
      automaton = BindingLHSAutomaton::new(save_index, automaton);
    }

    (automaton, subproblem_likely)
  }

  /// The theory-dependent part of `compile_lhs` called by `Term::compile_lhs(…)`.
  fn compile_lhs_aux(
    &mut self,
    match_at_top  : bool,
    variable_info : &VariableInfo,
    bound_uniquely: &mut NatSet,
  ) -> (BxLHSAutomaton, bool);

  fn compile_rhs(
    &mut self,
    rhs_builder    : &mut RHSBuilder,
    variable_info  : &mut VariableInfo,
    available_terms: &mut TermBag,
    eager_context  : bool,
  ) -> VariableIndex {
    if let Some((mut found_term, _)) = available_terms.find(self.as_ptr(), eager_context) {
      let found_term = found_term.deref_mut();

      if found_term.core_mut().save_index.is_none() {
        if let Some(variable_term) = found_term.as_any().downcast_ref::<VariableTerm>() {
          return variable_term.index.unwrap();
        }

        found_term.core_mut().save_index = Some(variable_info.make_protected_variable());
      }

      return found_term.core_mut().save_index.unwrap();
    }

    if let Some(variable_term) = self.as_any_mut().downcast_mut::<VariableTerm>() {
      let var_index = variable_term.index.unwrap();

      if variable_term.is_eager_context() {
        let index = variable_info.make_construction_index();
        rhs_builder.add_rhs_automaton(Box::new(CopyRHSAutomaton::new(var_index, index)));
        variable_term.core_mut().save_index = Some(index);
        available_terms.insert_built_term(self.as_ptr(), true);
        return index;
      }
      return var_index;
    }

    let index = self.compile_rhs_aux(rhs_builder, variable_info, available_terms, eager_context);
    self.core_mut().save_index = Some(index);
    available_terms.insert_built_term(self.as_ptr(), eager_context);

    index
  }

  /// The theory-dependent part of `compile_rhs` called by `Term::compile_rhs(…)`. Returns
  /// the `save_index`.
  fn compile_rhs_aux(
    &mut self,
    builder        : &mut RHSBuilder,
    variable_info  : &mut VariableInfo,
    available_terms: &mut TermBag,
    eager_context  : bool,
  ) -> VariableIndex;

  /// Compiles a term at the root/top level of a right-hand side. Unlike compileRhs(), this method guarantees creation of an automaton and handles term compilation specifically for top-level context.
  fn compile_top_rhs(
    &mut self,
    rhs_builder: &mut RHSBuilder,
    variable_info: &mut VariableInfo,
    available_terms: &mut TermBag,
  ) {
    let index = self.compile_rhs(rhs_builder, variable_info, available_terms, true);
    variable_info.use_index(index);
    // If we don't have any automata we must create one, if only to do the
    // replacement.
    if rhs_builder.is_empty() {
      rhs_builder.add_rhs_automaton(Box::new(TrivialRHSAutomaton::new(index)));
    }
  }

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
    let symbol     = self.symbol();
    let sort_table = symbol.sort_table();
    let kind       = sort_table.range_kind(); // should be const

    if symbol.arity().is_zero() {
      self.set_sort_info(kind, sort_table.traverse(0, SortIndex::ZERO)); // HACK
      return;
    }

    let mut step = SortIndex::ZERO;
    let mut seen_args_count = 0;

    for mut term in self.iter_args() {
      term.fill_in_sort_info();
      debug_assert_eq!(
        term.kind().unwrap(),
        symbol.domain_kind(seen_args_count),
        "component error on arg {} while computing sort of {}",
        seen_args_count,
        self.symbol()
      );
      step = sort_table.traverse(step.idx_unchecked(), term.core().sort_index);
      seen_args_count += 1;
    }

    // ToDo: Restore this assert.
    // debug_assert_eq!(seen_args_count, seen_args_count, "bad # of args for op");
    self.set_sort_info(kind, step);
  }

  /// Sets the kind and the sort index of this term.
  fn set_sort_info(&mut self, kind: KindPtr, sort_index: SortIndex) {
    let core        = self.core_mut();
    core.kind       = Some(kind);
    core.sort_index = sort_index;
  }

  fn sort(&self) -> Option<SortPtr> {
    if self.core().sort_index.is_index() {
      let sort_index = self.core().sort_index;
      if let Some(kind) = self.core().kind {
        return Some(kind.sort(sort_index));
      }
    }
    None
  }

  /// Recursively collects the terms in a set for structural sharing. This is the theory-specific
  /// part of `find_available_terms`.
  fn find_available_terms_aux(&self, available_terms: &mut TermBag, eager_context: bool, at_top: bool);

  /// Recursively collects the indices and occurs sets of this term and its descendants.
  fn index_variables(&mut self, indices: &mut VariableInfo) {
    // This condition needs to check an RcTerm for a VariableTerm
    if self.is_variable() {
      let index = indices.variable_to_index(self.as_ptr());
      let variable_term = self.as_any_mut().downcast_mut::<VariableTerm>().unwrap();

      // This call needs a mutable VariableTerm
      variable_term.index = Some(index);
      variable_term.occurs_below_mut().insert(index as usize);
    } else {
      // Accumulate in a local variable, because the iterator holds a mutable borrow.
      let mut occurs_below = NatSet::new();
      for mut arg in self.iter_args() {
        arg.index_variables(indices);
        // Accumulate the set of variables that occur under this symbol.
        occurs_below.union_in_place(&arg.occurs_below());
      }
      self.occurs_below_mut().union_in_place(&occurs_below);
    }
  }


  /// Recursively collects the terms in a set for structural sharing.
  ///
  /// This is a free function, because we want it wrapped in the Rc so that when we call `find_available_terms()`
  /// it's possible to add the Rc to the term set.
  fn find_available_terms(&self, available_terms: &mut TermBag, eager_context: bool, at_top: bool) {
    if self.ground() {
      return;
    }

    if !at_top {
      available_terms.insert_matched_term(self.as_ptr(), eager_context);
    }

    // Now do theory-specific stuff
    self.find_available_terms_aux(available_terms, eager_context, at_top);
  }

  // endregion Compiler-related Function

}


// region trait impls for Term

// ToDo: Revisit whether `structural_hash` is appropriate for the `Hash` trait.
// Use the `Term::structural_hash(…)` hash for `HashSet`s and friends.
impl Hash for dyn Term {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u32(self.structural_hash())
  }
}
// To use `Term` with `HashSet`, it needs to implement `Eq`
impl PartialEq for dyn Term {
  fn eq(&self, other: &Self) -> bool {
    self.structural_hash() == other.structural_hash()
  }
}
impl Eq for dyn Term {}

impl_display_debug_for_formattable!(dyn Term);
// endregion
