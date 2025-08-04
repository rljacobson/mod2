use std::any::Any;
use crate::{
  api::{
    automaton::RHSAutomaton,
    dag_node::{DagNodePtr, MaybeDagNode},
    dag_node_cache::DagNodeCache
    ,
    term::TermPtr
  },
  core::{
    substitution::Substitution,
    VariableInfo,
    VariableIndex
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

  fn construct(&self, matcher: &mut Substitution) -> DagNodePtr {
    let dag_node = self.term.dagify_aux(&mut DagNodeCache::default());
    matcher.bind(self.destination, Some(dag_node));
    dag_node
  }

  fn replace(&self, old: DagNodePtr, _matcher: &mut Substitution) -> DagNodePtr {
    self.term.overwrite_with_dag_node(old)
  }
}
