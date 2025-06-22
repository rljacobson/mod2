/*!

The [`NarrowingVariableInfo`] struct manages the state and mapping of variables during
narrowing operations. This struct maintains information about variables used in narrowing
processes, providing a bridge between variable indices and their DAG representations.

It is typically instantiated during the initialization of narrowing search states, such
as when creating a `NarrowingSearchState` object. The struct offers functionality to track
and manage variable bindings throughout the narrowing process, facilitating unification
and matching operations. [`NarrowingVariableInfo`] instances are commonly passed to
various narrowing-related functions and structs, serving as a central repository for
variable-related data and ensuring consistent variable handling across the narrowing system.

*/


use crate::api::{DagNodePtr, MaybeDagNode};


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
          if let Some(var) = v {
            // let var = unsafe { &**v };
            var.compare(variable).is_eq()
          } else {
            false
          }
        })
        .map(|i| i as i32)
  }
}
