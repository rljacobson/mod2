/*!

A `Kind` is a connected component of the lattice of sorts. `Kind`s <strike>need not be</strike> are not named, but a kind can be
represented by any of its `Sort`s.

See the module level documentation for the [`sort` module](crate::core::sort) for more
about sorts, kinds, and the subsort relation, and how they are represented in this codebase.

## Lifecycle and Ownership

`Kind`s are owned by the `Module` in which they are defined (whether implicitly or explicitly).
`Kind`s do not own their `Sort`s. As with the rest of the lattice infrastructure, once constructed,
`Kind`s are immutable and live as long as their owning `Module`, and as long their associated
`Sort`s. It is the responsibility of the owning `Module` to reclaim both `Kind`s and `Sort`s.


## Optimizations for Computing the Subsort Relation

See [the module level documentation](crate::core::sort), specifically the
section titled, "Optimizations for Computing a Subsort Relation at Runtime."


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


## See Also...

 1. `Kind`s are connected components of the graph of [`Sort`s](crate::core::sort::sort::Sort) induced by the subsort
    relation.
 2. A [`SortSpec`](crate::core::sort::sort_spec::SortSpec) is either a [`Sort`](crate::core::sort::sort::Sort) or a
    functor (from `SortSpec` to `SortSpec`).

*/


use std::{
  fmt::{
    Debug,
    Display
  }
};

use crate::{
  core::{
    sort::{
      sort::{
        SortPtr,
        SortPtrs
      },
      kind_error::KindError
    }
  }
};

// Convenience types
/// Each `Sort` holds a `KindPtr` to its `Kind`. However, it isn't clear if the `KindPtr` is ever dereferenced,
/// especially once the subsort relation is closed. Rather, `KindPtr` is just used as an identifier for the `Kind`.
pub type KindPtr = *mut Kind;
/// A Boxed kind to indicate owned heap-allocated memory.
pub type BxKind  = Box<Kind>;


pub struct Kind {
  /// The count of sorts that are maximal.
  pub maximal_sort_count: u32,
  /// Used during construction to detect cycles.
  pub visited_sort_count: u32,
  /// Is the `Kind` well-formed (acyclic)?
  pub error_free        : bool,
  pub sorts             : SortPtrs, // Sorts are owned by their parent module, not by their `Kind`.
}

impl Kind {
  /// Returns a boxed Kind.
  pub unsafe fn new(mut initial_sort: SortPtr) -> Result<BxKind, KindError> {

    let mut kind: BxKind = Box::new(
      Kind {
        error_free        : true,
        maximal_sort_count: 0,
        visited_sort_count: 0,
        sorts             : vec![initial_sort],
      }
    );
    let mut visited_sort_count: u32 = 0;

    // Recursively call `register_connected_sorts` on sub- and supersorts.
    kind.register_connected_sorts(initial_sort, &mut visited_sort_count);

    if kind.maximal_sort_count == 0 {
      // ToDo: Recording the error here might not be necessary considering we are returning the `Kind` wrapped in an error.
      kind.error_free = false;
      // Instead of marking the `Module` bad here, we return the constructed `Kind` wrapped in an error. The caller can
      // log the error.
      // log(Channel::Warning, 1, format!();
      // kind.sorts[0].get_module().mark_as_bad();
      return Err(
        KindError::NoMaximalSort {
          problem_sort: initial_sort,
          kind,
        }
      )
    }

    for i in 1..=kind.maximal_sort_count as usize {
      (*kind.sorts[0]).insert_subsort(kind.sorts[i]);
    }

    for i in 1..kind.sorts.len() {
      (*kind).process_subsorts((*kind).sorts[i]);
    }

    if kind.sorts.len() != visited_sort_count as usize {
      kind.error_free = false;
      return Err(
        KindError::CycleDetected {
          problem_sort: initial_sort,
          kind,
        }
      );
    }

    for i in (0..visited_sort_count).rev() {
      (*kind.sorts[i as usize]).compute_leq_sorts();
    }

    Ok(kind)
  }

  /// A helper function for computing the closure of the kind. The `visited_sort_count` is for cycle detection. If we visit more nodes (sorts) than we have, one of the nodes must have been visited twice..
  unsafe fn register_connected_sorts(&mut self, sort: SortPtr, visited_sort_count: &mut u32) {
    (*sort).kind = self;
    *visited_sort_count += 1;

    { // Visit subsorts
      let subsort_count = (*sort).subsorts.len();
      for i in 0..subsort_count {
        let s = (*sort).subsorts[i];
        if (*s).kind.is_null() {
          self.register_connected_sorts(s, visited_sort_count);
        }
      }
    }

    { // Visit supersorts
      let supersort_count = (*sort).supersorts.len();
      if supersort_count == 0 {
        (*sort).index_within_kind = self.append_sort(sort);
      } else {
        (*sort).unresolved_supersort_count = supersort_count;
        for i in 0..supersort_count {
          let s = (*sort).supersorts[i];
          if (*s).kind.is_null() {
            self.register_connected_sorts(s, visited_sort_count);
          }
        }
      }
    }
  }

  /// Auxiliary method to construct the sort lattice
  unsafe fn process_subsorts(&mut self, sort: SortPtr) {
    assert!(!sort.is_null(), "tried to process subsorts of a null porter to a sort");
    for subsort in (*sort).subsorts.iter() {
      assert!(!subsort.is_null(), "discovered a null subsort pointer");
      // We "resolve" `self` as a supersort for each of `self`'s subsorts. If `self` is the last unresolved supersort for the subsort, it is finally time to add the subsort to its kind. This ensures all supersorts of that subsort have been "resolved" before the subsort is added.
      (**subsort).unresolved_supersort_count -= 1;
      if (**subsort).unresolved_supersort_count == 0 {
        // Add the
        (**subsort).index_within_kind = self.append_sort(*subsort);;
      }
    }
  }

  /// Pushes the sort onto `self.sorts`, returning the index of the sort in `self.sorts`.
  pub fn append_sort(&mut self, sort: SortPtr) -> usize {
    self.sorts.push(sort);
    self.sorts.len() - 1
  }

}
