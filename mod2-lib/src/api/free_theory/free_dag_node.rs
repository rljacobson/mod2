use std::{
  any::Any,
  cmp::{
    max,
    Ordering
  },
  ops::DerefMut
};
use mod2_abs::hash::hash2;
use crate::{
  core::{
    gc::allocate_dag_node,
    dag_node_core::{
      DagNodeFlags,
      DagNodeCore,
      DagNodeFlag,
      ThinDagNodePtr
    },
    EquationalTheory,
    HashConsSet
  },
  api::{
    dag_node::{
      DagNodeVectorRefMut,
      DagNodeVector,
      DagNode,
      DagNodePtr,
      arg_to_dag_node,
      arg_to_node_vec
    },
    Arity,
    symbol::SymbolPtr,
    term::Term
  },
  HashType
};
use crate::api::dag_node::node_vec_to_args;
use crate::core::sort::SortIndex;

#[repr(transparent)]
pub struct FreeDagNode(DagNodeCore);

impl FreeDagNode {

  pub fn new(symbol: SymbolPtr) -> DagNodePtr {
    DagNodeCore::new(symbol)
  }

  pub fn with_args(symbol: SymbolPtr, args: &mut Vec<DagNodePtr>) -> DagNodePtr {
    let mut node = DagNodeCore::new(symbol);

    node.set_flags(DagNodeFlag::NeedsDestruction.into());
    node.core_mut().args = (DagNodeVector::from_slice(args) as *mut DagNodeVector) as *mut u8;

    node
  }

}

impl DagNode for FreeDagNode {
  #[inline(always)]
  fn as_any(&self) -> &dyn Any {
    self
  }

  #[inline(always)]
  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  #[inline(always)]
  fn as_ptr(&self) -> DagNodePtr {
    DagNodePtr::new(self as *const dyn DagNode as *mut dyn DagNode)
  }

  fn structural_hash(&self) -> HashType {
    let mut hash_value: HashType = self.symbol().hash();

    for arg in self.iter_args(){
      hash_value = hash2(hash_value, arg.structural_hash());
    }

    hash_value
  }

  #[inline(always)]
  fn core(&self) -> &DagNodeCore {
    &self.0
  }

  #[inline(always)]
  fn core_mut(&mut self) -> &mut DagNodeCore {
    &mut self.0
  }

  fn clear_copied_pointers_aux(&mut self) {
    for mut node in self.iter_args(){
      node.clear_copied_pointers()
    }
  }

  /// For hash consing, recursively checks child nodes to determine if a canonical copy needs to be made.
  ///
  /// Note: Does not add the canonical-ized self to `hash_cons_set`. This avoids an infinite recursion
  ///       in `HashConsSet::insert`.
  fn make_canonical(&self, hash_cons_set: &mut HashConsSet) -> DagNodePtr {
    // We only make a copy if one of the arguments is noncanonical.
    for (idx, d) in self.iter_args().enumerate() {
      let canonical_dag_node = hash_cons_set.insert(d);
      if DagNodePtr::addr_eq(&d, canonical_dag_node) {
        // The child node was already canonical.
        continue;
      }

      // Detected a non-canonical argument, need to make a new copy. It is convenient to
      // copy the arguments first. Everything up to `idx` is already canonical, so just
      // copy those to the new node.
      let mut new_args = Vec::with_capacity(self.arity().get() as usize);
      new_args.extend(self.iter_args().take(idx-1));
      // The `idx`'th argument was already made canonical.
      new_args.push(canonical_dag_node);
      // From `idx` on, we still need to make canonical.
      new_args.extend(self.iter_args().skip(idx).map(|d| hash_cons_set.insert(d)));

      // Now create a new copy of self with these args.
      let mut new_node  = FreeDagNode::with_args(self.symbol(), &mut new_args);
      // Copy over just the rewriting flags
      let rewrite_flags = self.flags() & DagNodeFlag::RewritingFlags;
      new_node.set_flags(rewrite_flags);
      new_node.set_sort_index(self.sort_index());

      return new_node;
    }

    // All args are already canonical, so we can use `self` as the canonical version.
    self.as_ptr()
  }

  fn make_canonical_copy(&self, hash_cons_set: &mut HashConsSet) -> DagNodePtr {
    // Almost identical to `FreeDagNode::make_clone()`.
    // It is convenient to copy the arguments first.
    let mut new_args = Vec::with_capacity(self.arity().get() as usize);
    new_args.extend(self.iter_args().map(|d| hash_cons_set.insert(d)));

    // Now create a new copy of self with these args.
    let mut new_node  = FreeDagNode::with_args(self.symbol(), &mut new_args);
    // Copy over just the rewriting flags
    let rewrite_flags = self.flags() & DagNodeFlag::RewritingFlags;
    new_node.set_flags(rewrite_flags);
    new_node.set_sort_index(self.sort_index());

    new_node
  }

  /// Constructs a shallow copy of this node.
  fn make_clone(&self) -> DagNodePtr {
    // It is convenient to copy the arguments first. We only copy the pointers; we don't clone them.
    let mut new_args = Vec::with_capacity(self.arity().get() as usize);
    new_args.extend(self.iter_args());

    // Now create a new copy of self with these args.
    let mut new_node  = FreeDagNode::with_args(self.symbol(), &mut new_args);
    // Copy over just the rewriting flags
    let rewrite_flags = self.flags() & DagNodeFlag::RewritingFlags;
    new_node.set_flags(rewrite_flags);
    new_node.set_sort_index(self.sort_index());

    new_node
  }

  fn copy_eager_upto_reduced_aux(&mut self) -> DagNodePtr {
    if self.len() > 0 {
      let node_vec = DagNodeVector::with_capacity(self.len());
      // ToDo: When strategies are implemented, everything might not be eager, so this code changes.
      node_vec.extend(self.iter_args().map(|mut node| node.copy_eager_upto_reduced_aux()));

      let mut new_node = DagNodeCore::new(self.symbol());
      new_node.core_mut().args = node_vec_to_args(node_vec);
      
      new_node
    } else {
      // A copy is just a DAG node with the same symbol
      DagNodeCore::new(self.symbol())
    }
  }

  fn compute_base_sort(&mut self) -> SortIndex {
    let symbol = self.symbol();
    // assert_eq!(self as *const _, subject.symbol() as *const _, "bad symbol");
    if symbol.arity().is_zero() {
      let idx = symbol.sort_table().traverse(0, SortIndex::ZERO);
      self.set_sort_index(idx); // Maude: HACK
      return idx;
    }

    let mut state = SortIndex::ZERO;
    // enumerate is only used for assertion
    for (idx, arg) in self.iter_args().enumerate() {
      let term_idx = arg.sort_index();
      assert_ne!(
        term_idx,
        SortIndex::UNKNOWN,
        "unknown sort encounter for arg {} subject = {}",
        idx,
        self as &dyn DagNode
      );
      state = symbol.sort_table().traverse(state.idx_unchecked(), term_idx);
    }
    self.set_sort_index(state);
    state
  }
}
