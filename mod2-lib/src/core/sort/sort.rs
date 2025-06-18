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

 - A ['Kind'](crate::core::sort::kind::Kind) is a connected component of the lattice of `Sort`s induced by the subsort
   relation.

*/

use std::{
  fmt::Write,
  ops::Deref
};
use mod2_abs::{
  NatSet,
  IString,
  UnsafePtr,
};
use crate::{
  api::{built_in::get_built_in_sort},
  core::{
    sort::{
      kind::KindPtr,
      sort_index::SortIndex
    },
    format::{FormatStyle, Formattable},
  },
  impl_display_debug_for_formattable,
};


/// A pointer to a sort. No ownership is assumed.
pub type SortPtr        = UnsafePtr<Sort>;
pub type MaybeSortIndex = Option<SortIndex>;


#[derive(Eq, PartialEq, Clone)]
pub struct Sort {
  pub name: IString,
  /// The `index_within_kind` is the index of the sort within its `Kind`.
  ///
  /// The value `unresolved_supersort_count` is only used during `Kind` construction. Only when all
  /// supersorts have been assigned an `index_within_kind` can this `Sort`'s `index_within_kind`
  /// be assigned, which only occurs when `unresolved_supersort_count` reaches zero. Therefore,
  /// we also use this field for `unresolved_supersort_count` as an optimization for subsort computations.
  pub index_within_kind: SortIndex,

  /// This is the index for which all sorts with `index >= fast_compare_index` are subsorts.
  fast_compare_index: SortIndex,

  /// Adjacency lists, generally only immediately adjacent sorts. Besides sorts that
  /// are subsorts (resp supersorts) via transitivity, there may be sorts within the
  /// connected component that are incomparable to this one and thus neither a super- nor
  /// sub-sort. The transitive closure of `<=` is computed and stored in `leq_sorts`.
  pub subsorts  : Vec<SortPtr>,
  pub supersorts: Vec<SortPtr>,
  /// Holds the indices within kind of sorts that are subsorts of this sort, including transitively.
  // ToDo: If `subsorts`/`supersorts` aren't used after construction, don't store them in `Sort`. It looks like
  //       `supersorts` is not but `subsorts` might be.
  pub leq_sorts :  NatSet,

  /// The connected component this sort belongs to.
  pub kind: Option<KindPtr>,
}

// This is an abomination. See `api/built_in/mod.rs`.
unsafe impl Send for Sort {}
unsafe impl Sync for Sort {}

impl Default for Sort {
  fn default() -> Self {
    Sort {
      name                      : IString::default(),
      index_within_kind         : SortIndex::default(), // Also used for `unresolved_supersort_count` during kind construction
      fast_compare_index        : SortIndex::default(),
      subsorts                  : Vec::<SortPtr>::default(),
      supersorts                : Vec::<SortPtr>::default(),
      leq_sorts                 : NatSet::default(),
      kind                      : None,
    }
  }
}

impl Sort {
  pub fn any() -> SortPtr {
    unsafe{ get_built_in_sort("Any").unwrap_unchecked() }
  }

  pub fn none() -> SortPtr {
    unsafe{ get_built_in_sort("None").unwrap_unchecked() }
  }

  pub fn new(name: IString) -> Sort {
    Sort{
      name,
      ..Self::default()
    }
  }

  /// Antisymmetrically inserts `other` as a subsort of `self` and `self` as a supersort of `other`.
  pub fn insert_subsort(&mut self, mut other: SortPtr) {
    self.subsorts.push(other);
    other.supersorts.push(UnsafePtr::new(self));
  }

  /// Compute the transitive closure of the subsort relation as stored in `self.leq_sorts`.
  ///
  /// This only works if this method is called on each sort in the connected component in increasing order. This is
  /// guaranteed by how `sort.register_connected_sorts` is called. Used during subsort relation closure, during `Kind`
  /// construction.
  pub fn compute_leq_sorts(&mut self) {
    self.leq_sorts.insert(self.index_within_kind.idx_unchecked());
    for subsort in self.subsorts.iter() {
      let subsort_leq_sorts: &NatSet = &subsort.leq_sorts;
      self.leq_sorts.union_in_place(subsort_leq_sorts);
    }

    // Now determine `fast_compare_index`, the index for which all sorts with `index >= fast_compare_index` are subsorts.
    self.fast_compare_index = self.index_within_kind;
    let total_sort_count    = unsafe {self.kind.unwrap_unchecked().sorts.len()};
    for i in (self.index_within_kind.idx_unchecked()..total_sort_count).rev() {
      if !self.leq_sorts.contains(i) {
        self.fast_compare_index = SortIndex::try_from(i + 1).unwrap();
        break;
      }
    }
  }
  
  /// Determines if self <= other. 
  #[inline(always)]
  pub fn leq(&self, other: SortPtr) -> bool {
    other.leq_sorts.contains(self.index_within_kind.idx_unchecked())
  }
}


impl Formattable for Sort {
  fn repr(&self, out: &mut dyn Write, _style: FormatStyle) -> std::fmt::Result {
    if self.index_within_kind == SortIndex::ZERO {
      write!(out, "[{}]", self.name)
    } else { 
      write!(out, "{}", self.name)
    }
  }
}

impl_display_debug_for_formattable!(Sort);



#[inline(always)]
pub fn index_leq_sort(index: SortIndex, sort: &Sort) -> bool {
  assert_ne!(index, SortIndex::UNKNOWN, "unknown sort");
  if index >= sort.fast_compare_index {
    true
  } else { 
    sort.leq_sorts.contains(index.idx_unchecked())
  }
}

#[inline(always)]
pub fn sort_leq_index(sort: &Sort, index: SortIndex) -> bool {
  index_leq_sort(
    sort.index_within_kind,
    sort.kind
        .unwrap()
        .sort(index)
        .deref(),
  )
}
