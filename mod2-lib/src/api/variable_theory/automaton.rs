use crate::{
  api::{
    automaton::LHSAutomaton,
    dag_node::DagNodePtr,
    subproblem::MaybeSubproblem
  },
  core::{
    sort::SortPtr,
    substitution::Substitution
  }
};
use crate::api::MaybeExtensionInfo;
use crate::core::VariableIndex;

pub struct VariableLHSAutomaton {
  index: VariableIndex,
  sort: SortPtr,
  copy_to_avoid_overwriting: bool,
}

impl VariableLHSAutomaton {
  pub fn new(index: VariableIndex, sort: SortPtr, copy_to_avoid_overwriting: bool) -> Self {
    VariableLHSAutomaton {
      index,
      sort,
      copy_to_avoid_overwriting,
    }
  }
}


impl LHSAutomaton for VariableLHSAutomaton {
  fn match_(&mut self, subject: DagNodePtr, solution: &mut Substitution, extension_info: MaybeExtensionInfo) -> (bool, MaybeSubproblem) {
    self.match_variable(
      subject,
      self.index,
      self.sort,
      self.copy_to_avoid_overwriting,
      solution,
      extension_info
    )
  }
}
