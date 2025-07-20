/*!

A state transition graph with hash consing.

*/

mod graph;
mod index;
mod position_state;

use mod2_abs::{HashMap, HashSet};
use crate::{
  api::DagNodePtr,
  core::{
    pre_equation::PreEquationPtr as RulePtr,
    StateGraphIndex
  }
};

pub use graph::StateTransitionGraph;
pub use index::*;
pub use position_state::*;

type ArcMap = HashMap<u32, HashSet<RulePtr>>;

struct State{
  hash_cons_index: u32,
  state_dag      : DagNodePtr,
  parent         : StateGraphIndex,
  next_states    : Vec<u32>,
  //rewrite_state: Option<RewriteSearchState>,
  forward_arcs   : ArcMap,
  fully_explored : bool,
}

