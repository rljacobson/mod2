/*!

A `Sort` is a named type. `Sort`s can be related to each other via a subsort relation, which in the
absence of error conditions is a partial order.

See the module level documentation for the [`sort`](crate::core::sort) for more about
sorts, kinds, and the subsort relation, and how they are represented in this codebase.

## Lifecycle and Ownership

Sorts are owned by the `Module` in which they are defined, *not* by a `Kind` or adjacency list. Once
the subsort lattice is constructed (that is, the `Kind`s and the adjacency lists in the `Sort`s),
it is immutable for the lifetime of the sorts (equivalently, for the lifetime of the `Module`).

## Optimizations for Computing the Subsort Relation

See [the module level documentation](crate::core::sort), specifically the
section titled, "Optimizations for Computing a Subsort Relation at Runtime."

## See Also...

 - A [`SortSpec`](crate::core::sort::sort_spec::SortSpec) is a generalization of `Sort` that additionally permits
   functors.
 - A ['Kind'](crate::core::sort::kind::Kind) is a connected component of the lattice of `Sort`s induced by the subsort
   relation.

*/

use std::{
  cell::RefCell,
  rc::Rc
};
use std::fmt::Display;
use crate::{
  abstractions::{
    IString,
    NatSet
  },
  core::sort::kind::{
    Kind,
    KindPtr
  },
};

/// A pointer to a sort. No ownership is assumed.
pub type SortPtr  = *mut Sort;
/// A vector of pointers to `Sort`s. No ownership is assumed.
pub type SortPtrs = Vec<SortPtr>;

#[derive(Clone)]
pub struct Sort {
  pub name:       IString,
  /// The `index_within_kind` is the index of the sort within its `Kind`.
  // ToDo: As it is only computed after subsort closure and `unresolved_supersort_count` is only used during subsort closure,
  //       we could also use this field for `unresolved_supersort_count` as an optimization for subsort computations.
  pub index_within_kind: usize,


  /// This is the index for which all sorts with `index >= fast_compare_index` are subsorts.
  fast_compare_index: usize,

  /// Only used during `Kind` construction to compute `unresolved_supersort_count`. Only when all
  /// supersorts have been assigned an `index_within_kind` can this `Sort`'s `index_within_kind`
  /// be assigned, which only occurs when `unresolved_supersort_count` reaches zero.
  pub unresolved_supersort_count: usize,

  /// Adjacency lists, generally only immediately adjacent sorts. Besides sorts that
  /// are subsorts (resp supersorts) via transitivity, there may be sorts within the
  /// connected component that are incomparable to this one and thus neither a super- nor
  /// sub-sort. The transitive closure of `<=` is computed and stored in `leq_sorts`.
  pub subsorts  : SortPtrs,
  pub supersorts: SortPtrs,
  /// Holds the indices within kind of sorts that are subsorts of this sort, including transitively.
  // ToDo: If `subsorts`/`supersorts` aren't used after construction, don't store them in `Sort`. It looks like
  //       `supersorts` is not but `subsorts` might be.
  pub leq_sorts :  NatSet,

  // The connected component this sort belongs to.
  pub kind: KindPtr, // This should be a weak reference
}

impl Default for Sort {
  fn default() -> Self {
    Sort {
      name                      : IString::default(),
      index_within_kind         : 0,
      unresolved_supersort_count: 0, // Only used during kind construction until `index_within_kind` is determined.
      fast_compare_index        : 0,
      subsorts                  : SortPtrs::default(),
      supersorts                : SortPtrs::default(),
      leq_sorts                 : NatSet::default(),
      kind                      : std::ptr::null_mut(),
    }
  }
}

impl Sort {
  pub fn new(name: IString) -> Sort {
    Sort{
      name,
      ..Self::default()
    }
  }

  /// Returns -1 for special sort Any, -2 for special sort None, and 0 otherwise.
  pub fn arity(&self) -> i16 {
    match self.name  {
      v if v == IString::from("Any")  => -1,
      v if v == IString::from("None") => -2,
      _ =>  0
    }
  }


  /// Antisymmetrically inserts `other` as a subsort of `self` and `self` as a supersort of `other`.
  pub fn insert_subsort(&mut self, other: SortPtr) {
    assert!(!other.is_null(), "other sort is null pointer");
    self.subsorts.push(other);
    unsafe {
      (*other).supersorts.push(self);
    }
  }

  /// Compute the transitive closure of the subsort relation as stored in `self.leq_sorts`.
  ///
  /// This only works if this method is called on each sort in the connected component in increasing order. This is
  /// guaranteed by how `sort.register_connected_sorts` is called. Used during subsort relation closure, during `Kind`
  /// construction.
  pub fn compute_leq_sorts(&mut self) {
    self.leq_sorts.insert(self.index_within_kind);
    for subsort in self.subsorts.iter() {
      let subsort_leq_sorts: &NatSet = unsafe { &(**subsort).leq_sorts };
      self.leq_sorts.union_in_place(subsort_leq_sorts);
    }

    // Now determine `fast_compare_index`, the index for which all sorts with `index >= fast_compare_index` are subsorts.
    self.fast_compare_index = self.index_within_kind;
    let total_sort_count    = unsafe {(*self.kind).sorts.len()};
    for i in (self.index_within_kind..total_sort_count).rev() {
      if !self.leq_sorts.contains(i) {
        self.fast_compare_index = i + 1;
        break;
      }
    }
  }
}

impl Display for Sort {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name)
  }
}
