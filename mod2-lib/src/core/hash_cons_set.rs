/*!

A `HashConsSet` implements structural sharing of subnodes in a DAG.

The primary purpose of `HashConsSet` is to ensure that structurally identical DAG nodes are
represented by a single canonical instance in memory.

1. reduces memory usage by eliminating duplicate representations;
2. allows structural equality checks to be performed by simple pointer comparison;
3. provides a foundation for memoization of operations on terms.

*/

use crate::{
  api::DagNodePtr,
  core::gc::root_container::RootMap
};

pub struct HashConsSet {
  node_set: RootMap
}

impl HashConsSet {
  /// Inserts the node if a canonical version of it is not in the set. Returns the canonical node.
  pub fn insert(&mut self, node: DagNodePtr) -> DagNodePtr {
    if let Some(&canon) = self.node_set.get(&node.structural_hash()) {
      // Fast path: a canonical node already exists
      let mut canonical = canon;
      canonical.upgrade_sort_index(node);

      canonical
    } else {
      // Slow path: make a canonical node from `node`
      let canonical_node = node.make_canonical(self);
      assert_eq!(canonical_node.structural_hash(), node.structural_hash());
      self.node_set.insert(canonical_node.structural_hash(), canonical_node);

      canonical_node
    }
  }

  /// Inserts a canonical copy of the node if it is not in the set. Returns the canonical node. Does not upgrade the
  /// canonical node's sort index. We assume the sort is either unknown or unimportant.
  // ToDo: Why are we not upgrading the canonical node's sort?
  pub fn insert_copy(&mut self, node: DagNodePtr) -> DagNodePtr {
    if let Some(&canonical_node) = self.node_set.get(&node.structural_hash()) {
      // Fast path: a canonical node already exists
      canonical_node
    } else {
      // Slow path: make a canonical node from `node`
      let canonical_node = node.make_canonical_copy(self);
      assert_eq!(canonical_node.structural_hash(), node.structural_hash());
      self.node_set.insert(canonical_node.structural_hash(), canonical_node);

      canonical_node
    }
  }

  /// Fetches the canonical version of the given node, but does not insert if it's not found.
  pub fn get_canonical(&self, node: DagNodePtr) -> Option<&DagNodePtr> {
    self.node_set.get(&node.structural_hash())
  }
}
