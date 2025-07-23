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

type ArcMap = HashMap<u32, HashSet<RulePtr>>;

struct State{
  hash_cons_index: u32,
  state_dag      : DagNodePtr,
  parent         : StateGraphIndex,
  next_states    : Vec<u32>,
  rewrite_state  : Option<RewriteSearchState>,
  forward_arcs   : ArcMap,
  fully_explored : bool,
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
