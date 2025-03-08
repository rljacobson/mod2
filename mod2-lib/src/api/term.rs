/*!

Every theory's term type must implement the `Term` trait. The concrete term type should
have a `TermCore` member that can be accessed through the trait method `Term::core()`
and `Term::core_mut()`. This allows a lot of shared implementation in `TermCore`.

*/

use std::{
  any::Any,
  fmt::{Display, Formatter},
  hash::{Hash, Hasher},
  cmp::Ordering,
  collections::{
    HashMap,
    hash_map::Entry
  },
  sync::atomic::Ordering::Relaxed
};

use mod2_abs::{
  NatSet,
  RcCell
};

use crate::{
  api::{
    dag_node::{DagNodePtr, DagNode},
    UNDEFINED,
    symbol::{
      Symbol,
      SymbolPtr,
      SymbolSet
    }
  },
  core::{
    dag_node_core::{
      DagNodeCore,
      DagNodeFlag,
    },
    format::{
      FormatStyle,
      Formattable
    },
    term_core::{
      cache_node_for_term,
      clear_cache_and_set_sort_info,
      lookup_node_for_term,
      TermAttribute,
      TermCore
    },
    substitution::Substitution
  }
};

pub type BxTerm    = Box<dyn Term>;
pub type MaybeTerm = Option<&'static dyn Term>;
pub type RcTerm    = RcCell<dyn Term>;
pub type TermSet   = HashMap<u32, usize>;

pub trait Term: Formattable {
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  fn as_ptr(&self) -> *const dyn Term;
  fn semantic_hash(&self) -> u32;
  /// Normalizes the term, returning the computed hash and `true` if the normalization changed
  /// the term or `false` otherwise.
  fn normalize(&mut self, full: bool) -> (u32, bool);



  fn core(&self) -> &TermCore;
  fn core_mut(&mut self) -> &mut TermCore;

  /// This method should construct a new `Term` of the concrete implementing type,
  /// including its `TermCore` member, and return it wrapped in an `Rc`.
  // fn new(symbol: SymbolPtr) -> BxTerm;

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
  fn iter_args(&self) -> Box<dyn Iterator<Item = &dyn Term> + '_>;
  // Implement an empty iterator with:
  //    Box::new(std::iter::empty::<&dyn Term>())

  #[inline(always)]
  fn symbol_ref(&self) -> &'static Symbol {
    self.core().symbol_ref()
  }

  #[inline(always)]
  fn symbol(&self) -> SymbolPtr {
    self.core().symbol
  }

  /// Compute the number of nodes in the term tree
  fn compute_size(&self) -> i32 {
    let cached_size = &self.core().cached_size;

    if cached_size.get() != UNDEFINED {
      cached_size.get()
    }
    else {
      let mut size = 1; // Count self.
      for arg in self.iter_args() {
        size += arg.compute_size();
      }

      cached_size.set(size);
      size
    }
  }

  // endregion Accessors


  // region Comparison Functions

  fn compare_term_arguments(&self, _other: &dyn Term) -> Ordering;
  fn compare_dag_arguments(&self, _other: &dyn DagNode) -> Ordering;

  #[inline(always)]
  fn compare_dag_node(&self, other: &dyn DagNode) -> Ordering {
    if self.symbol_ref().hash() == other.symbol_ref().hash() {
      self.compare_dag_arguments(other)
    } else {
      self.symbol_ref().compare(other.symbol_ref())
    }
  }

  fn partial_compare(&self, partial_substitution: &mut Substitution, other: &dyn DagNode) -> Option<Ordering> {
    if !self.is_stable() {
      // Only used for `VariableTerm`
      return self.partial_compare_unstable(partial_substitution, other);
    }

    if std::ptr::addr_eq(self.symbol(), other.symbol()) {
      // Only used for `FreeTerm`
      return self.partial_compare_arguments(partial_substitution, other);
    }

    if self.symbol_ref().compare(other.symbol_ref()) == Ordering::Less {
      Some(Ordering::Less)
    } else {
      Some(Ordering::Greater)
    }
  }

  #[inline(always)]
  fn compare(&self, other: &dyn Term) -> Ordering {
    let r = self.symbol_ref().compare(other.symbol_ref());
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

  // endregion


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
    let semantic_hash = self.semantic_hash();
    if let Some(dag_node) = lookup_node_for_term(semantic_hash) {
      return dag_node;
    }

    let dag_node = self.dagify_aux();
    cache_node_for_term(semantic_hash, dag_node);

    dag_node
  }

  /// Create a directed acyclic graph from this term. This method has the implementation-specific stuff.
  fn dagify_aux(&self) -> DagNodePtr;

  // endregion

}


// region trait impls for Term

// ToDo: Revisit whether `semantic_hash` is appropriate for the `Hash` trait.
// Use the `Term::compute_hash(…)` hash for `HashSet`s and friends.
impl Hash for dyn Term {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u32(self.semantic_hash())
  }
}

impl PartialEq for dyn Term {
  fn eq(&self, other: &Self) -> bool {
    self.semantic_hash() == other.semantic_hash()
  }
}

impl Eq for dyn Term {}
// endregion


impl Display for dyn Term {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "[{}]", self.symbol_ref())
  }
}
