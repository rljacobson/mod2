/*!

A state transition graph with hash consing.

*/

use mod2_abs::{HashMap, HashSet};
use crate::api::DagNodePtr;
use crate::core::{pre_equation::PreEquationPtr as RulePtr, rewriting_context::BxRewritingContext, HashConsSet, StateGraphIndex};

type ArcMap = HashMap<u32, HashSet<RulePtr>>;

pub struct State{
  hash_cons_index: u32,
  state_dag      : DagNodePtr,
  parent         : u32,
  next_states    : Vec<u32>,
  //rewrite_state: Option<RewriteSearchState>,
  forward_arcs   : ArcMap,
  fully_explored : bool,
}

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
      seen: Vec::new(),
      hash_cons_to_seen: Vec::new(),
      hash_cons_set: HashConsSet::new()
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
