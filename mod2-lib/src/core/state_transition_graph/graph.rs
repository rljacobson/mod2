use crate::{
  api::DagNodePtr,
  core::{
    HashConsSet,
    StateGraphIndex,
    rewriting_context::BxRewritingContext,
    state_transition_graph::State
  }
};

pub struct StateTransitionGraph {
  pub(crate) initial_context: BxRewritingContext,
  seen                      : Vec<State>,
  hash_cons_to_seen         : Vec<usize>,
  hash_cons_set             : HashConsSet,
}

impl StateTransitionGraph {
  pub fn new(initial_context: BxRewritingContext) -> StateTransitionGraph {
    StateTransitionGraph{
      initial_context,
      seen             : Vec::new(),
      hash_cons_to_seen: Vec::new(),
      hash_cons_set    : HashConsSet::new()
    }
  }

  pub fn state_count(&self) -> usize {
    self.seen.len()
  }

  pub fn get_state_dag(&self, state_idx: StateGraphIndex) -> DagNodePtr {
    // self.hash_cons_set.get_canonical(self.seen[state_idx.idx()].hash_cons_index)
    self.seen[state_idx.idx()].state_dag
  }

  pub fn get_next_state(&self, _state_idx: StateGraphIndex, _index: StateGraphIndex) -> StateGraphIndex {
    todo!("Implement get_next_state")
  }
}
