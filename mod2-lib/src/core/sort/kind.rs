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

 - `Kind`s are connected components of the graph of [`Sort`s](crate::core::sort::sort::Sort) induced by the subsort
    relation.


*/


use std::{
  fmt::{
    Formatter,
    Debug,
    Display
  },
  ops::Deref
};
use mod2_abs::{heap_construct, join_iter, UnsafePtr};

use crate::{
  core::{
    sort::{
      SortPtr,
      KindError,
      Sort,
    },
    SortIndex
  }
};

// Convenience types
/// Each `Sort` holds a `KindPtr` to its `Kind`. However, it isn't clear if the `KindPtr` is ever dereferenced,
/// especially once the subsort relation is closed. Rather, `KindPtr` is just used as an identifier for the `Kind`.
pub type KindPtr = UnsafePtr<Kind>;
/// A Boxed kind to indicate owned heap-allocated memory.
pub type BxKind  = Box<Kind>;

pub struct Kind {
  /// The count of sorts that are maximal.
  pub maximal_sort_count: u32,
  /// Is the `Kind` well-formed (acyclic)?
  pub error_free        : bool,
  pub sorts             : Vec<SortPtr>, // Sorts are owned by their parent module, not by their `Kind`.
}

impl Kind {
  /// Returns a boxed Kind.
  pub fn new(initial_sort: SortPtr) -> Result<BxKind, KindError> {
    let mut kind: BxKind = Box::new(
      Kind {
        error_free        : true,
        maximal_sort_count: 0,
        sorts             : vec![],
      }
    );


    /*
    We walk the sorts graph, as determined by the adjacency lists in the sorts,
    adding any new sorts we visit to the kind.
    */
    // Keep count of sorts in kind to detect cycles
    let mut visited_sort_count: u32 = 0;

    // Save initial sort so that we have a name for the component and its error sort.
    // The error sort of each component is added to the module.

    let mut error_sort = SortPtr::new(heap_construct!(Sort::new(initial_sort.name.clone())));

    // Recursively call `register_connected_sorts` on sub- and supersorts.
    kind.register_connected_sorts(error_sort, &mut visited_sort_count);
    kind.register_connected_sorts(initial_sort, &mut visited_sort_count);

    if visited_sort_count == 1 {
      // ToDo: Recording the error here might not be necessary considering we are returning the `Kind` wrapped in an
      //       error.

      // The error is that the connected component in the sort graph that contains `initial_sort` has no maximal sorts
      // due to a cycle.
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

    // Make every sort in the kind a subsort of the error sort.
    for i in 1..=kind.maximal_sort_count as usize {
      error_sort.insert_subsort(kind.sorts[i]);
    }

    // Process subsorts. Length of `kind.sorts` may increase.
    {
      let mut i = 0;
      loop {
        if i >= kind.sorts.len() { break; }
        kind.process_subsorts(kind.sorts[i]);
        i += 1
      }
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

    // Now that the entire connected component is included in the Kind, complete the
    // transitive closure of the subsort relation.
    for i in (0..visited_sort_count).rev() {
      kind.sorts[i as usize].compute_leq_sorts();
    }

    Ok(kind)
  }

  /// A helper function for computing the closure of the kind. The `visited_sort_count` is for cycle detection. If we visit more nodes (sorts) than we have, one of the nodes must have been visited twice.
  fn register_connected_sorts(&mut self, mut sort: SortPtr, visited_sort_count: &mut u32) {
    sort.kind = Some(KindPtr::new(self));
    *visited_sort_count += 1;

    { // Visit subsorts
      let subsort_count = sort.subsorts.len();
      for i in 0..subsort_count {
        let s = sort.subsorts[i];
        if s.kind.is_none() {
          self.register_connected_sorts(s, visited_sort_count);
        }
      }
    }

    { // Visit supersorts
      let supersort_count = sort.supersorts.len();
      if supersort_count == 0 {
        sort.index_within_kind = self.append_sort(sort);
      } else {
        sort.index_within_kind = SortIndex::from_usize(supersort_count);
        for &s in sort.supersorts.iter() {
          if s.kind.is_none() {
            self.register_connected_sorts(s, visited_sort_count);
          }
        }
      }
    }
  }

  /// Auxiliary method to construct the sort lattice
  fn process_subsorts(&mut self, mut sort: SortPtr) {
    for subsort in sort.subsorts.iter_mut() {
      // We "resolve" `self` as a supersort for each of `self`'s subsorts. If
      // `self` is the last unresolved supersort for the subsort, it is finally
      // time to add the subsort to its kind. This ensures all supersorts
      // of that subsort have been "resolved" before the subsort is added.
      subsort.index_within_kind -= 1;
      if subsort.index_within_kind == SortIndex::Zero {
        // All supersorts resolved, so add to kind. There is a symmetric statement
        // for subsorts in `Kind::register_connected_sorts`
        subsort.index_within_kind = self.append_sort(*subsort);
      }
    }
  }

  /// Pushes the sort onto `self.sorts`, returning the index of the sort in `self.sorts`.
  pub fn append_sort(&mut self, sort: SortPtr) -> SortIndex {
    self.sorts.push(sort);
    SortIndex::from_usize(self.sorts.len() - 1)
  }

  // region Accessors

  pub fn sort_count(&self) -> usize {
    self.sorts.len()
  }

  pub fn sort(&self, idx: SortIndex) -> SortPtr {
    self.sorts[idx.idx()]
  }

  // endregion Accessors
}

impl Display for Kind {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let iter = self.sorts.iter().map(|sort_ptr| sort_ptr.to_string());
    write!(f, "{{{}}}", join_iter(iter, |_| ", ".to_string()).collect::<String>())
  }
}

impl Debug for Kind {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let iter = self.sorts.iter().map(|sort_ptr| sort_ptr.name.deref());
    write!(f, "Kind{{{}}}", join_iter(iter, |_| ", ").collect::<String>())
  }
}
