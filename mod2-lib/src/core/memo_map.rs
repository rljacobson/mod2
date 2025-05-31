/*!

A `MemoMap` is a convenience wrapper around a `HashConsSet` for memoization of a module's symbols.

A `MemoMap` memoizes a function $f(x) = y$, where $x$ and $y$ are `DagNode`s. We call $x$ the from-dag
and $y$ the to-dag.

Maude's hash cons set assigns an index to each dag node, and Maude's `MemoMap` gives out that index for
both from-dag and to-dag nodes. In Maude, the `MemoMap` is owned by the module, and additional functionality
is provided by `MemoTable`. In our implementation, we combine `MemoTable` with `MemoMap`.

*/

use crate::{
  api::dag_node::DagNodePtr,
  core::{
    gc::root_container::{RootMap, RootVec},
    HashConsSet,
    rewriting_context::RewritingContext
  }
};


pub struct MemoMap {
  /// The to-dags are always canonical.
  dags: HashConsSet,
  /// Maps from-dags to to-dags
  dag_map: RootMap
}

impl MemoMap {

  /// If a memoized result exists, overwrites the subject with a clone of the cached result,
  /// increments the equation count in the context, and returns `true`. If no memoized
  /// result exists, appends the subject’s index to the `sourceSet` and returns `false`.
  // ToDo: Overwriting in-place is really problematic with fat pointers. 
  pub fn memo_rewrite(&mut self, source_set: &mut RootVec, subject: &mut DagNodePtr, context: &mut RewritingContext) -> bool {
    if let Some(mut canonical_to_dag) = self.get_to_dag(*subject) {
      if context.is_trace_enabled() {
        // ToDo: Implement tracing in MemoMap
        todo!("MemoMap tracing not implemented yet.")
        // context.trace_pre_equation_rewrite(subject, 0, RewriteType::Memoized);
        // if context.trace_abort() {
        //   return false;
        // }
      }
      *subject = canonical_to_dag.overwrite_with_clone(*subject);
      context.equation_count += 1;
      if context.is_trace_enabled() {
        // context.trace_post_eq_rewrite(subject);
      }
      true
    } else {
      source_set.push(*subject);
      false
    }
  }
  
  /// Inserts a mapping `from_dag` ↦ `to_dag`.
  fn assign_to_dag(&mut self, from_dag: DagNodePtr, to_dag: DagNodePtr) {
    let canonical_to_dag = self.dags.insert(to_dag);
    self.dag_map.insert(from_dag.structural_hash(), canonical_to_dag);
  }

  /// Get the `to_dag` for the provided `from_dag`.
  fn get_to_dag(&self, from_dag: DagNodePtr) -> Option<DagNodePtr> {
    self.dag_map.get(&from_dag.structural_hash()).copied()
  }

  /// Makes the given `dag_node` canonical. This can be used to canonicalize from-dag nodes.
  fn canonicalize(&mut self, dag_node: DagNodePtr) {
    self.dags.insert(dag_node);
  }
}
