/*!

A `Kind` is a connected component of the lattice of sorts. `Kind`s need not be named, but a kind can be represented by
any of its `Sort`s.

## Error States During Kind Construction

A sort is considered "maximal" if there are no other sorts that are a supersort (parent or ancestor sort) of it. Such a
sort is at the top of the hierarchy within a component. There can be more than one. However, it's possible to have no
maximal sort in a connected component if there's a cycle in the sort graph, as none of the sorts in the cycle can be
considered a maximal sort because they all have another sort above them in the cycle. The existence of a cycle is an
error state.

Recall that a connected graph is acyclic if and only if it has $n-1$ edges, where $n$ is the number of its nodes. (Such
a graph is, of course, a tree.) We use the proof of this fact as a poor man's cycle detection during `Kind` construction
by keeping track of how many nodes we visit. If we visit more than the total number of nodes, the pigeonhole principle
demands that we must have encountered the same node more than once.

We report two kinds of errors during construction of a kind:
 1. a cycle detected by the lack of maximal sorts (or really any sorts), and
 2. a cycle detected due to pigeonhole principle (failure to linear order the sorts).

*/


use std::{
  fmt::{Display, Formatter},
  error::Error,
  rc::{Rc, Weak},
  fmt::Debug
};

use crate::abstractions::{Channel, log};
use crate::core::sort::sort::{RcSort, WeakSorts};

// Public convenience types
pub type RcKind  = Rc<Kind>;
pub type WkKind  = Weak<Kind>;
pub type PtrKind = *Kind;

pub enum KindError {
  CycleDetected {
    problem_sort: RcSort,
    kind: Kind
  },
  NoMaximalSort {
    problem_sort: RcSort,
    kind: Kind
  }
}
impl Display for KindError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self{

      KindError::CycleDetected { problem_sort, .. } => {
        write!(
          f,
          "the connected component in the sort graph that contains sort {} has no maximal sorts due to a cycle.",
          problem_sort.borrow().name
        )
      } // end `KindError::CycleDetected` branch

      KindError::NoMaximalSort { problem_sort, .. } => {
        write!(
          f,
          "the connected component in the sort graph that contains sort \"{}\" has no maximal sorts due to a cycle.",
          problem_sort.borrow().name
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


pub struct Kind {
  /// The count of sorts that are maximal.
  pub maximal_sort_count: u32,
  /// Is the `Kind` well-formed (acyclic)?
  pub error_free        : bool,
  pub sorts             : WeakSorts, // Sorts are owned by their parent module.
}

impl Kind {
  pub fn new(initial_sort: RcSort) -> Result<Self, KindError> {

    let mut kind = Kind {
      error_free        : true,
      maximal_sort_count: 0,
      sorts             : vec![initial_sort.downgrade()],
    };

    // Recursively calls `register_connected_sorts` on sub- and supersorts.
    initial_sort.register_connected_sorts(&kind);



    if kind.maximal_sort_count == 0 {
      kind.error_free = false;
      // log(Channel::Warning, 1, format!();
      kind.sorts[0].get_module().mark_as_bad();
      return kind
    }

    for i in 1..=kind.maximal_sort_count as usize {
      kind.sorts[0].insert_subsort(kind.sorts[i].clone());
    }

    for i in 1..kind.sorts.len() {
      kind.sorts[i].process_subsorts();
    }

    if kind.sorts.len() != kind.sort_count as usize {
      kind.error_free = false;
      println!("the connected component in the sort graph that contains sort {} could not be linearly ordered due to a cycle.", kind.sorts[0].id());
      kind.sorts[0].get_module().mark_as_bad();
      return kind
    }

    for i in (0..kind.sort_count).rev() {
      kind.sorts[i as usize].compute_leq_sorts();
    }

    kind
  }

  fn register_connected_sorts(&mut self, sort: &mut RcSort, sorts_encountered_count: &mut u16) {
    sort.sort_component = self;
    *sorts_encountered_count += 1;

    // explore subsorts
    let nr_sorts = sort.subsorts.len();
    for i in 0..nr_sorts {
      let s = &mut sort.subsorts[i];
      if s.sort_component.is_none() {
        self.register_connected_sorts(s);
      }
    }

    // explore supersorts
    let nr_sorts = sort.supersorts.len();
    if nr_sorts == 0 {
      sort.sort_index = self.append_sort(sort);
    } else {
      sort.nr_unresolved_supersorts = nr_sorts;
      for i in 0..nr_sorts {
        let s = &mut sort.supersorts[i];
        if s.sort_component.is_none() {
          self.register_connected_sorts(s);
        }
      }
    }
  }


}
