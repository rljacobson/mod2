/*!

A state transition graph with hash consing.

*/

mod graph;
mod index;
mod position_state;
mod rewrite_search_state;

use enumflags2::{bitflags, BitFlags};
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
use rewrite_search_state::RewriteSearchState;

type ArcMap = HashMap<StateGraphIndex, HashSet<RulePtr>>;

struct State{
  // pub hash_cons_index: u32,
  pub state_dag      : DagNodePtr,
  pub parent         : StateGraphIndex,
  pub next_states    : Vec<StateGraphIndex>,
  pub rewrite_state  : Option<RewriteSearchState>,
  pub forward_arcs   : ArcMap,
  pub fully_explored : bool,
}

#[bitflags]
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateFlag {
  // Position State Flags
  RespectFrozen,       // 1
  SetUnstackable,      // 128
  RespectUnstackable,  // 1024
  ExtensionInfoValid,

  // Rewrite State Flags
  AllowNonexec,        // 32
  SetUnrewritable,     // 256
  RespectUnrewritable, // 2048
  WithExtension,       // A bool on RewriteSearchState

  // Search State Flags
  GCContext,           // 2 Delete context in our dtor
  GCSubstitution,      // 4 Delete initial substitution (if there is one) in our dtor
  IgnoreCondition,     // 8 Ignore conditions of conditional PreEquations
}

pub type StateFlags = BitFlags<StateFlag>;
