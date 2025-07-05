/*!

A kind of "dummy" RHS automata used, for example, to just do a substitution.

*/


use std::any::Any;

use crate::{
  core::{
    substitution::{MaybeDagNode, Substitution},
    VariableInfo,
    VariableIndex
  },
  api::{DagNodePtr, RHSAutomaton},
};

#[derive(Copy, Clone, Default)]
pub(crate) struct TrivialRHSAutomaton {
  index: VariableIndex,
}

impl TrivialRHSAutomaton {
  pub fn new(index: VariableIndex) -> Self {
    Self { index }
  }
}

impl RHSAutomaton for TrivialRHSAutomaton {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn remap_indices(&mut self, variable_info: &mut VariableInfo) {
    self.index = variable_info.remap_index(self.index);
  }

  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode {
    matcher.value(self.index)
  }

  fn replace(&self, old: DagNodePtr, matcher: &mut Substitution) -> DagNodePtr {
    matcher.value(self.index)
           .unwrap()
           .overwrite_with_clone(old)
  }
}
