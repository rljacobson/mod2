/*!

A `MemoMap` is a convenience wrapper around a `HashConsSet` for memoization.

A `MemoMap` memoizes a function $f(x) = y$, where $x$ and $y$ are `DagNode`s. We call $x$ the from-dag
and $y$ the to-dag.

Maude's hash cons set assigns an index to each dag node, and Maude's `MemoMap` gives out that index for 
both from-dag and to-dag nodes.

*/

use crate::{
  core::{
    gc::root_container::RootMap,
    HashConsSet
  },
  api::dag_node::DagNodePtr,
};

pub struct MemoMap {
  /// The to-dags are always canonical.
  dags: HashConsSet,
  /// Maps from-dags to to-dags
  dag_map: RootMap
}

impl MemoMap {
  /// Inserts a mapping `from_dag` ↦ `to_dag`.
  pub fn assign_to_dag(&mut self, from_dag: DagNodePtr, to_dag: DagNodePtr) {
    let canonical_to_dag = self.dags.insert(to_dag);
    self.dag_map.insert(from_dag.structural_hash(), canonical_to_dag);
  }
  
  /// Get the `to_dag` for the provided `from_dag`.
  pub fn get_to_dag(&self, from_dag: DagNodePtr) -> Option<DagNodePtr> {
    self.dag_map.get(&from_dag.structural_hash()).copied()
  }
  
  /// Makes the given `dag_node` canonical. This can be used to canonicalize from-dag nodes.
  pub fn canonicalize(&mut self, dag_node: DagNodePtr) {
    self.dags.insert(dag_node);
  }
}
