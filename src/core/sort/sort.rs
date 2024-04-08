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
  /// The `index_within_kind` is the index of the sort within its `Kind`. It is used with `fast_compare_index` as an optimization for subsort computations..
  pub index_within_kind: usize,


  /// This is the index for which all sorts with `index >= fast_compare_index` are subsorts.
  fast_compare_index: usize,

  /// Only used during `Kind` construction to compute `fast_compare_index`. Only when all
  /// supersorts have been assigned an `index_within_kind` can this `Sort`'s `index_within_kind`
  /// be assigned, which only occurs when `unresolved_supersort_count` reaches zero.
  pub unresolved_supersort_count: usize,


  pub subsorts  : SortPtrs, // There may be sorts within the connected component that are
  pub supersorts: SortPtrs, // incomparable to this one and thus neither a super- nor sub-sort.
  /// Holds the indices within kind of sorts that are subsorts of this sort.
  pub leq_sorts :  NatSet,

  // The connected component this sort belongs to.
  pub kind: KindPtr, // This should be a weak reference
}

impl Default for Sort {
  fn default() -> Self {
    Sort {
      name                      : IString::default(),
      index_within_kind         : 0,
      fast_compare_index        : 0,
      unresolved_supersort_count: 0,
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

  /* This method was moved into `Kind`
  /// Sets the `Sort`'s `Kind` to the given kind, then recursively calls `register_connected_sorts` on sub- and
  /// supersorts that do not yet have their kind set.
  ///
  /// This method is called during `Kind` closure.
  pub(crate) unsafe fn register_connected_sorts(&mut self, kind: *mut Kind) {
    assert!(!kind.is_null(), "tried to register connected sorts with null pointer to Kind");

    self.kind = kind;
    unsafe {
      (*kind).visited_sort_count += 1;
    }

    {
      let subsort_count = self.subsorts.len();
      for i in 0..subsort_count {
        let s = self.subsorts[i];
        if (*s).kind.is_null() {
          (*s).register_connected_sorts(kind);
        }
      }
    }

    {
      let supersort_count = self.supersorts.len();
      if supersort_count == 0 {
        self.sort_index = (*kind).append_sort(self);
      } else {
        self.unresolved_supersort_count = supersort_count;
        for i in 0..supersort_count {
          let s = self.supersorts[i];
          if (*s).kind.is_null() {
            (*s).register_connected_sorts(kind);
          }
        }
      }
    }
  }
  */


  /// Antisymmetrically inserts `other` as a subsort of `self` and `self` as a supersort of `other`.
  /// Used during subsort relation closure, during `Kind` construction.
  pub fn insert_subsort(&mut self, other: SortPtr) {
    assert!(!other.is_null(), "other sort is null pointer");
    self.subsorts.push(other);
    unsafe {
      (*other).supersorts.push(self);
    }
  }

  /// Used during subsort relation closure, during `Kind` construction. Constructs `self.leq_sorts`.
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
