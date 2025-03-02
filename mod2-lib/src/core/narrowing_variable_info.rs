/*!


*/


use crate::{core::substitution::MaybeDagNode, api::dag_node::DagNodePtr};


pub struct NarrowingVariableInfo {
  variables: Vec<MaybeDagNode>,
}

impl NarrowingVariableInfo {
  #[inline(always)]
  pub(crate) fn variable_count(&self) -> usize {
    self.variables.len()
  }

  #[inline(always)]
  pub(crate) fn index_to_variable(&self, index: usize) -> MaybeDagNode {
    if let Some(d) = self.variables.get(index) {
      d.clone()
    } else {
      None
    }
  }

  // ToDo: Use a BiMap instead of using `Vec::position`, which is O(n).
  pub(crate) fn variable_to_index(&mut self, variable: DagNodePtr) -> i32 {
    let idx = self.variable_to_index_without_insert(variable);
    match idx {
      Some(i) => i,
      None => {
        self.variables.push(Some(variable.clone()));
        (self.variables.len() - 1) as i32
      }
    }
  }

  #[inline(always)]
  pub(crate) fn iter(&self) -> Box<dyn Iterator<Item = (usize, DagNodePtr)> + '_> {
    Box::new(self.variables.iter().filter_map(|v| (*v).clone()).enumerate())
  }

  #[inline(always)]
  pub(crate) fn variable_to_index_without_insert(&mut self, variable: DagNodePtr) -> Option<i32> {
    // assert!(variable != &VariableTerm::default(), "null term");
    self.variables
        .iter()
        .position(|v| {
          if let Some(v) = v {
            let var = unsafe { &**v };
            var.compare(variable).is_eq()
          } else {
            false
          }
        })
        .map(|i| i as i32)
  }
}
