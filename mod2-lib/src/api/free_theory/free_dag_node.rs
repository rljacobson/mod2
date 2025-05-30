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

#[repr(transparent)]
pub struct FreeDagNode(DagNodeCore);

impl FreeDagNode {

  pub fn new(symbol: SymbolPtr) -> DagNodePtr {
    DagNodeCore::with_theory(symbol, EquationalTheory::Free)
  }

  pub fn with_args(symbol: SymbolPtr, args: &mut Vec<DagNodePtr>) -> DagNodePtr {
    let mut node = DagNodeCore::with_theory(symbol, EquationalTheory::Free);

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
      let mut new_args = Vec::with_capacity(self.arity().as_numeric() as usize);
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
    let mut new_args = Vec::with_capacity(self.arity().as_numeric() as usize);
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
    let mut new_args = Vec::with_capacity(self.arity().as_numeric() as usize);
    new_args.extend(self.iter_args());

    // Now create a new copy of self with these args.
    let mut new_node  = FreeDagNode::with_args(self.symbol(), &mut new_args);
    // Copy over just the rewriting flags
    let rewrite_flags = self.flags() & DagNodeFlag::RewritingFlags;
    new_node.set_flags(rewrite_flags);
    new_node.set_sort_index(self.sort_index());

    new_node
  }
}
