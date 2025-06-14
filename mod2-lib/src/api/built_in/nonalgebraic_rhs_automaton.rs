use std::any::Any;
use crate::{
  core::{
    VariableIndex,
    VariableInfo,
    substitution::Substitution
  },
  api::{
    term::TermPtr,
    dag_node_cache::DagNodeCache,
    dag_node::{DagNodePtr, MaybeDagNode},
    built_in::{
      nonalgebraic_lhs_automaton::NonalgebraicLHSAutomaton,
      NADataType
    },
    automaton::RHSAutomaton
  },
};

pub struct NonalgebraicRHSAutomaton {
  pub term       : TermPtr,
  pub destination: VariableIndex
}

impl NonalgebraicRHSAutomaton {
  pub fn new(term: TermPtr, destination: VariableIndex) -> Box<Self> {
    Box::new(Self{ term, destination })
  }
}

impl RHSAutomaton for NonalgebraicRHSAutomaton {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn remap_indices(&mut self, variable_info: &mut VariableInfo) {
    variable_info.remap_index(self.destination);
  }

  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode {
    let dag_node = self.term.dagify_aux(&mut DagNodeCache::default());
    matcher.bind(self.destination, Some(dag_node));
    Some(dag_node)
  }

  fn replace(&mut self, old: DagNodePtr, _matcher: &mut Substitution) -> DagNodePtr {
    self.term.overwrite_with_dag_node(old)
  }
}
