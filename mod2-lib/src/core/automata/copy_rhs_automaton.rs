/*!

Right hand side automata that make copies of bindings in the substitution.


*/


use std::any::Any;
use mod2_abs::debug;
use crate::{
  api::{
    RHSAutomaton,
    DagNodePtr, 
    MaybeDagNode
  },
  core::{
    substitution::Substitution,
    VariableInfo
  }
};
use crate::core::VariableIndex;

pub struct CopyRHSAutomaton {
  original_index: VariableIndex,
  copy_index:     VariableIndex,
}

impl CopyRHSAutomaton {
  pub fn new(original_index: VariableIndex, copy_index: VariableIndex) -> Self {
    // TODO: Are these indices necessarily positive?
    Self {
      original_index,
      copy_index,
    }
  }
}


impl RHSAutomaton for CopyRHSAutomaton {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn remap_indices(&mut self, variable_info: &mut VariableInfo) {
    self.original_index = variable_info.remap_index(self.original_index);
    self.copy_index = variable_info.remap_index(self.copy_index);
  }

  /*
  fn record_info(&self, compiler: &mut StackMachineRhsCompiler) -> bool {
    let mut sources = Vec::new();
    sources.push(self.original_index);
    compiler.record_function_eval(0, self.copy_index, sources);
    true
  }
  */

  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode {
    let orig = matcher.value(self.original_index);
    if let Some(mut orig_dag_node) = orig {
      debug!(
        2,
        "CopyRhsAutomaton::construct {}",
        orig_dag_node
      );

      let new_dag_node = orig_dag_node.copy_eager_upto_reduced();
      orig_dag_node.clear_copied_pointers();
      matcher.bind(self.copy_index, new_dag_node.clone());
      new_dag_node
    } else {
      unreachable!("No DagNode for original index. This is a bug.");
    }
  }

  fn replace(&mut self, old: DagNodePtr, matcher: &mut Substitution) -> DagNodePtr {
    let orig = matcher.value(self.original_index);

    if let Some(mut orig_dag_node) = orig {
      let new_dag_node = orig_dag_node.copy_eager_upto_reduced();
      orig_dag_node.clear_copied_pointers();

      if let Some(mut new_dag_node) = new_dag_node {
        return new_dag_node.overwrite_with_clone(old);
      }
    }
    unreachable!("No DagNode for original index. This is a bug.");
  }
}
