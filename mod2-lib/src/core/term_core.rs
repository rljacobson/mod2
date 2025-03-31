/*!

A `Term` is a node in the expression tree. That is, an expression tree is a term, and
each subexpression is a term. The algorithms do not operate on expression trees (terms).
Instead, the algorithms operate on a directed acyclic graph (DAG) is constructed from the
tree. Thus, for each `Term` type, there is a corresponding `DagNode` type. However, because
of structural sharing, the node instances themselves are not in 1-to-1 correspondence.

*/

use std::{
  cell::Cell,
  collections::hash_map::Entry,
  sync::atomic::{
    Ordering::Relaxed,
    AtomicBool,
  },
  ops::Deref
};

use enumflags2::{bitflags, BitFlags};
use once_cell::sync::Lazy;

use mod2_abs::{NatSet, HashMap};

use crate::{
  api::{
    UNDEFINED,
    symbol::{Symbol, SymbolPtr, SymbolSet},
    dag_node::DagNodePtr
  },
  core::sort::kind::KindPtr
};

pub type TermSet   = HashMap<u32, usize>;

static mut CONVERTED_TERMS: Lazy<TermSet> =  Lazy::new(|| {
  TermSet::new()
});

static mut SUBDAG_CACHE : Lazy<Vec<DagNodePtr>> = Lazy::new(|| {
  Vec::new()
});

static SET_SORT_INFO_FLAG: AtomicBool = AtomicBool::new(false);


#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TermType {
  Free,
  Bound,
  Ground,
  NonGround,
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TermAttribute {
  ///	A subterm is stable if its top symbol cannot change under instantiation.
  Stable,

  ///	A subterm is in an eager context if the path to its root contains only
  ///	eagerly evaluated positions.
  EagerContext,

  ///	A subterm "honors ground out match" if its matching algorithm guarantees
  ///	never to return a matching subproblem when all the terms variables
  ///	are already bound.
  HonorsGroundOutMatch,
}

pub type TermAttributes = BitFlags<TermAttribute, u8>;

#[derive(Clone)]
pub struct TermCore {
  /// The top symbol of the term
  pub(crate) symbol: SymbolPtr,
  /// The handles (indices) for the variable terms that occur in this term or its descendants
  pub(crate) occurs_set      : NatSet,
  pub(crate) context_set     : NatSet,
  // ToDo: How is this related to the kind of its symbol?
  pub(crate) kind            : Option<KindPtr>,
  pub(crate) collapse_symbols: SymbolSet,
  pub(crate) attributes      : TermAttributes,
  pub(crate) sort_index      : i32,
  pub(crate) term_type       : TermType,
  pub(crate) save_index      : i32,            // NoneIndex = -1
  hash_value                 : u32,

  /// The number of nodes in the term tree
  pub(crate) cached_size:  Cell<i32>,
}

impl TermCore {
  pub fn new(symbol: SymbolPtr) -> TermCore {
    TermCore {
      symbol,
      occurs_set      : Default::default(),
      context_set     : Default::default(),
      kind            : None,                 // ToDo: Initialize to `symbol.range_kind()` ?
      collapse_symbols: Default::default(),
      attributes      : TermAttributes::default(),
      sort_index      : UNDEFINED,
      term_type       : TermType::Free,
      save_index      : 0,
      hash_value      : 0,
      cached_size     : Cell::new(UNDEFINED),
    }
  }

  // region Accessors

  /// Is the term stable?
  #[inline(always)]
  pub fn is_stable(&self) -> bool {
    self.attributes.contains(TermAttribute::Stable)
  }

  /// A subterm "honors ground out match" if its matching algorithm guarantees never to return a matching subproblem
  /// when all the terms variables are already bound.
  #[inline(always)]
  pub fn honors_ground_out_match(&self) -> bool {
    self.attributes.contains(TermAttribute::HonorsGroundOutMatch)
  }

  #[inline(always)]
  pub fn set_honors_ground_out_match(&mut self, value: bool) {
    if value {
      self.attributes.insert(TermAttribute::HonorsGroundOutMatch);
    } else {
      self.attributes.remove(TermAttribute::HonorsGroundOutMatch);
    }
  }

  #[inline(always)]
  pub fn is_eager_context(&self) -> bool {
    self.attributes.contains(TermAttribute::EagerContext)
  }

  #[inline(always)]
  pub fn is_variable(&self) -> bool {
    self.symbol.is_variable()
  }

  #[inline(always)]
  pub fn ground(&self) -> bool {
    self.occurs_set.is_empty()
  }

  /// The handles (indices) for the variable terms that occur in this term or its descendants
  #[inline(always)]
  pub(crate) fn occurs_below(&self) -> &NatSet {
    &self.occurs_set
  }

  #[inline(always)]
  pub(crate) fn occurs_below_mut(&mut self) -> &mut NatSet {
    &mut self.occurs_set
  }

  #[inline(always)]
  pub(crate) fn occurs_in_context(&self) -> &NatSet {
    &self.context_set
  }

  #[inline(always)]
  pub(crate) fn occurs_in_context_mut(&mut self) -> &mut NatSet {
    &mut self.context_set
  }

  #[inline(always)]
  pub(crate) fn collapse_symbols(&self) -> &SymbolSet {
    &self.collapse_symbols
  }

  #[inline(always)]
  pub fn symbol(&self) -> SymbolPtr {
    self.symbol
  }

  #[inline(always)]
  pub fn symbol_ref(&self) -> &dyn Symbol {
    self.symbol.deref()
  }

  // endregion Accessors

}

/// This function is called from `Term::term_to_dag()`.
pub fn clear_cache_and_set_sort_info(set_sort_info: bool) {
  SET_SORT_INFO_FLAG.store(set_sort_info, Relaxed);
  unsafe { #[allow(static_mut_refs)] SUBDAG_CACHE.clear(); }
  unsafe { #[allow(static_mut_refs)] CONVERTED_TERMS.clear(); }
}

/// This free function plays the role of `Term::dagify()`. The sub DAG cache implements structural
/// sharing.
pub fn lookup_node_for_term(semantic_hash: u32) -> Option<DagNodePtr> {
  if let Entry::Occupied(occupied_entry) = unsafe{ #[allow(static_mut_refs)] CONVERTED_TERMS.entry(semantic_hash) } {
    let idx = *occupied_entry.get();
    Some(unsafe{ SUBDAG_CACHE[idx] })
  } else {
    None
  }
}

/// This free function (along with the one above) plays the role of `Term::dagify()`.
/// The sub DAG cache implements structural sharing.
pub fn cache_node_for_term(semantic_hash: u32, node: DagNodePtr) {
  let idx = unsafe{ #[allow(static_mut_refs)] SUBDAG_CACHE.len() };
  unsafe{ #[allow(static_mut_refs)] SUBDAG_CACHE.push(node) };
  // sub_dags.insert(self_hash, d.clone());
  unsafe{ #[allow(static_mut_refs)] CONVERTED_TERMS.insert(semantic_hash, idx) };
}
