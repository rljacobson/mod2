/*!

A `CachedDag` lazily converts and caches `Term` objects as `DagNode` objects,
supporting normalization, eager path preparation, and stack machine compilation.

Built-in symbols use `CachedDag` for common results like `true`,
`false`, and `notFound`, converting them to DAG nodes only when needed.

*/

use mod2_abs::NatSet;
use crate::api::{BxTerm, DagNodePtr, TermPtr};
use crate::core::gc::root_container::{BxRootVec, RootVec};


pub struct CachedDag {
  term                : BxTerm,
  dag_node_root       : Option<BxRootVec>,
  // instruction_sequence: Vec<Instruction>,
}

impl CachedDag {
  pub fn new(term: BxTerm) -> Self {
    Self {
      term,
      dag_node_root: None,
    }
  }

  // region accessors

  #[inline(always)]
  pub fn term(&self) -> TermPtr {
    self.term.as_ptr()
  }

  #[inline(always)]
  pub fn set_term(&mut self, term: BxTerm) {
    self.term          = term;
    self.dag_node_root = None;
  }

  pub fn dag_node(&mut self) -> DagNodePtr {
    match &self.dag_node_root {

      None => {
        let set_sort_info  = self.term.core().sort_index.is_index();
        let dag_node       = self.term.term_to_dag(set_sort_info);
        self.dag_node_root = Some(RootVec::with_node(dag_node));
        dag_node
      }

      Some(root_vec) => root_vec.node()

    }
  }

  #[inline(always)]
  pub fn reset(&mut self) {
    self.dag_node_root = None;
  }

  // endregion

  /// Returns true if the term changed.
  pub fn normalize(&mut self) -> bool {
    let (maybe_term, mut changed, _new_hash) = self.term.normalize(true);
    if let Some(term) = maybe_term {
      self.set_term(term);
      changed = true;
    }
    changed
  }

  // Maude's prepare() method
  pub fn mark_eager_arguments(&mut self) {
    let empty_eager_variables       = NatSet::new();
    let mut empty_problem_variables = Vec::new();
    self.term.mark_eager_arguments(0, &empty_eager_variables, &mut empty_problem_variables);
  }
}
