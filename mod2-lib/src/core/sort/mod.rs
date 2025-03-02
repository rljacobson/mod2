/*!

A sort (represented in code by the [`Sort`](crate::core::sort::Sort) struct) is a named type.
Sorts are organized into a lattice structure defined by a subsort relation. In this lattice,
sorts (types) are connected in a hierarchical manner, where one sort can be a subsort (subtype) of
another, representing a specific kind of relationship between them. Connected components in this
context, which are called *kinds* (represented by the [`Kind`](crate::core::sort::kind::Kind)
struct), refer to the maximal sets of sorts that are connected through these subsort relations,
essentially forming distinct groups or clusters within the lattice.

## Lifecycle and Ownership

`Sort`s and `Kind`s, once constructed, are owned by the `Module` in which they are
defined, are immutable, and share the lifetime of their parent `Module`, which has the
responsibility of destroying them (and freeing their memory). The subsort relation
is represented by adjacency lists stored in the sorts themselves: each sort stores
a list of (pointers to) its subsorts and a list of (pointers to) its supersorts.

## The Subsort Relation

Any two sorts are either incomparable, or equal, or one sort is a subsort of the other.
While the adjacency lists in the `Sort`s are by themselves sufficient to compute the subsort
relation, several optimizations are employed. First, each `Sort` knows its `Kind`. If two sorts
belong to different kinds, they are incomparable. Next, during construction of each `Kind` the
`Kind`'s sorts are assigned an index such that for two `Sort`s `x` and `y`, `x.sort_index <=
y.sort_index` implies that `x` is not a supersort of `y`. For each `Sort` `y`, we precompute
the smallest index such that `x.sort_index <= index` implies that `y` is subsort of `x`.
This gives fast computation of the subsort relation for `Kind`s that are "nearly" linear.

## Computing the Closure of the Subsort Relation

Sorts and their subsort relationships are not specified to the system as, say, a complete
adjacency list. Rather, the smallest partially ordered set containing the specified relations
has to be computed. Mathematically this is called the closure of the relation, and this closure
must be computed during the construction of `Sort`s and `Kind`s. Computing the closure
of the subsort relation involves identifying all implicit subsort relationships that are
not directly specified but are inferred from the given relations. In the context of sorts
and kinds, this process ensures that the system comprehensively understands all possible
hierarchical relationships among sorts, even those not explicitly defined. If A is a subsort
of B, and B is a subsort of C, then through transitive closure, A is also a subsort of C. This
transitivity needs to be captured in the system's internal representation of sorts and kinds.

Malformed specifications are possible. In particular, it is possible to specify subsort relations
that introduce a cycle, for example, by specifying that A < B, B < C, and C < A for distinct
sorts A, B, and C. In these cases the closure computation is terminated and a warning is issued.

Computing the closure of the subsort relation only happens once at the time
of construction of the `Kind`s. Computation of any given relation between to
sorts can then use this existing precomputed infrastructure during runtime.

## Optimizations for Computing a Subsort Relation at Runtime

A couple of optimizations are used for the computation of the subsort
relationship between two given sorts during runtime. These optimizations use
precomputation performed during the construction of the `Kind`s and involves
the implementation of both `Kind` and `Sort`. We describe this precomputation here.

### Sort Indexing and the Subsort Relation

The `index_within_kind` property of `Sort` provides a numeric index that helps to efficiently navigate the sort hierarchy. The key principle is that if `x.index_within_kind > y.index_within_kind`, then `x` is never a supersort of `y`. This ordering facilitates an optimization: the value `Sort.fast_compare_index` is set to the smallest `index_within_kind` such that `x.index_within_kind >= fast_compare_index` ensures `x` is a subsort of `y`. When this comparison can be made, the adjacency lists do not need to be searched, significantly speeding up the comparison operation. For sort heirarchies that are "nearly" or completely linear, this can be a significant savings.

### Role of `Sort.unresolved_supersort_count`

The `unresolved_supersort_count` property of `Sort` acts as a counter for the number of supersorts of a given sort that have yet to be explored or registered within their `Kind` (connected component). This property ensures that a sort's `index_within_kind` is only finalized once all of its supersorts have been accounted for, thus maintaining the correct hierarchical ordering within the connected component, namely that a `Sort`'s `index_within_kind` is greater than that of all of its supersorts. When the `unresolved_supersort_count` of a sort drops to zero, only then is its `index_within_kind` assigned, ensuring that all its supersorts have already been processed and have a lower `index_within_kind`. (Note that this does not necessarily mean every `Sort` having a lower `index_within_kind` is a supersort, as there may be that some incomporable `Sort` that has a lower `index_within_kind`.)

### Depth-First Search in Registering Sorts

The methods `Kind::register_connected_sorts` and `Sort::process_subsorts` implement a depth-first search through the sort lattice. This approach guarantees that each sort *and its subsorts* are fully explored and registered in the connected component before moving to the next sort. This systematic traversal is essential for correctly establishing the `index_within_kind` for each `Sort` and for maintaining the subsort-supersort relationships accurately within each `Kind`.

### The `Sort::compute_leq_sorts` Method and `fast_compare_index` Calculation

The `Sort::compute_leq_sorts` method is used to build a `leq_sorts` set, which includes all sorts that are less than or equal to a given sort, based on the `index_within_kind`. The `fast_compare_index` is an optimization that identifies the smallest `index_within_kind` for which any sort with an index greater or equal is guaranteed to be less than or equal to the current sort. The calculation of `fast_compare_index` cannot simply assume it to be equal to `index_within_kind`, because there may be noncomparable sorts with indices greater than `index_within_kind`.

### During Runtime

To compare two sorts `A` and `B` during runtime:

1. if they have different `Kind`s, they are incomparable;
2. if `A.index_within_kind >= B.fast_compare_index`, then `A` is a subsort of `B`;  if `B.index_within_kind >= A.fast_compare_index`, then `B` is a subsort of `A`;
3. otherwise, check whether either `A.subsorts.contains(B)` or `A.supersorts.contains(B)` (equivalently with roles of `A` and `B` swapped). This is the slow path.


 */

pub mod kind;
pub mod sort;
pub mod sort_spec;
pub mod collection;
pub(crate) mod kind_error;

pub use sort::*;
