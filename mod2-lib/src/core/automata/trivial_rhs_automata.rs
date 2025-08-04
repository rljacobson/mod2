/*!

A kind of "dummy" RHS automata used, for example, to just do a substitution.

*/


use std::any::Any;

use crate::{
  core::{
    substitution::Substitution,
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

  fn construct(&self, matcher: &mut Substitution) -> DagNodePtr {
    // ToDo: This appears to be the only implementation that can conceivably return a null pointer.
    //       If this ever happens in practice, this unwrap is illegitimate, and `construct` needs to
    //       return a `MaybeDagNode`.
    matcher.value(self.index).unwrap()
  }

  fn replace(&self, old: DagNodePtr, matcher: &mut Substitution) -> DagNodePtr {
    matcher.value(self.index)
           .unwrap()
           .overwrite_with_clone(old)
  }
}
