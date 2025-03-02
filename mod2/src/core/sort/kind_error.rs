/*!

When computing the closure of the subsort relation, encountering a cycle is an error condition.

*/

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::core::sort::kind::BxKind;
use crate::core::sort::SortPtr;

pub enum KindError {
  CycleDetected {
    problem_sort: SortPtr,
    kind        : BxKind
  },
  NoMaximalSort {
    problem_sort: SortPtr,
    kind        : BxKind
  }
}

impl Display for KindError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self{

      KindError::CycleDetected { problem_sort, .. } => {
        write!(
          f,
          "the connected component in the sort graph that contains sort {} could not be linearly ordered due to a cycle.",
          unsafe{ &(**problem_sort).name }
        )
      } // end `KindError::CycleDetected` branch

      KindError::NoMaximalSort { problem_sort, .. } => {
        write!(
          f,
          "the connected component in the sort graph that contains sort \"{}\" has no maximal sorts due to a cycle.",
          unsafe{ &(**problem_sort).name }
        )
      }

    } // end match on `KindError`

  }
}

impl Debug for KindError {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    Display::fmt(self, f)
  }
}

impl Error for KindError{}
