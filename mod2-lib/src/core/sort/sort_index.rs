/*!

`SortIndex` is a special indexing value for sorts that combines unknown/impossible and
uninitialized/error states with "valid" index states that can index into arrays.

# Associated Constants

`SortIndex` has associated constants for all special values
(corresponding roughly to Maude's values–see next section):

| Special Sort Value                 | Value | Context                                                      |
| ---------------------------------- | ----- | ------------------------------------------------------------ |
| `SortIndex::KIND`                  | 0     | Top sort in a connected component; used in `Sort` and for kind-level declarations in `SortTable` |
| `SortIndex::ERROR`                 | 0     | Error sort (same as `KIND`); used as starting point in sort computation and error checking (`SortTable`) |
| `SortIndex::FIRST_USER_SORT`       | 1     | Starting index for user-defined sorts; used in `Sort`        |
| `SortIndex::UNKNOWN`               | None  | Indicates uncomputed/unknown sort; used in runtime sort checking, memory allocation, and assertions before traverse operations (`Symbol`) |
| `SortIndex::UNINITIALIZED`         | 0     | Local variable in sort diagram construction; used in `SortTable` |
| `SortIndex::IMPOSSIBLE`            | None  | Local variable indicating impossible state in sort diagram construction; used in `SortTable` |
| `SortIndex::FAST_CASE_UNIQUE_SORT` | None  | Fast symbol handling when no unique sort exists; used in `Symbol` |
| `SortIndex::SLOW_CASE_UNIQUE_SORT` | 0     | Slow symbol handling when no unique sort exists; used in `Symbol` |

# Implementation

The `SortIndex` type is a wrapper around `Option<NonZero<u16>>`, which has the
same size as `u16`. Internally the `None` value is represented by 0, and the values
`0..u16::MAX` are represented internally by `1..=u16::MAX`. The value `u16::MAX`
is not representable. We will always qualify with the word internal when we refer
to the internal value; the word value alone is the index value being represented.

# Background in Maude's codebase

In Maude, sort indices have type `int`, which is a signed 32
bit integer. The `Sort` class defines these special values:

- `KIND = 0` - represents the kind (top sort in a connected component)
- `ERROR_SORT = 0` - same value as KIND, used for error sorts  
- `FIRST_USER_SORT = 1` - the starting index for user-defined sorts
- `SORT_UNKNOWN = -1` - indicates an unknown/uncomputed sort

In the `SortTable` class, during sort diagram construction, there are local special indices:

- `UNINITIALIZED = 0` - indicates an uninitialized state
- `IMPOSSIBLE = -1` - indicates an impossible/invalid state


| Special Sort Value | Value | Context |
|-------------------|-------|---------|
| `Sort::KIND` | 0 | Top sort in a connected component; used in (sort.hh:42) and for kind-level declarations in (sortTable.cc:168) |
| `Sort::ERROR_SORT` | 0 | Error sort (same as KIND); used as starting point in sort computation (sortTable.cc:369) and error checking (sortTable.cc:199)  |
| `Sort::FIRST_USER_SORT` | 1 | Starting index for user-defined sorts; used in (sort.hh:44)  |
| `Sort::SORT_UNKNOWN` | -1 | Indicates uncomputed/unknown sort; used in runtime sort checking, memory allocation (AU_StackSort.cc:32) , and assertions before traverse operations (symbol.cc:496) |
| `SpecialSortIndices::UNINITIALIZED` | 0 | Local variable in sort diagram construction; used in  (sortTable.cc:272) |
| `SpecialSortIndices::IMPOSSIBLE` | -1 | Local variable indicating impossible state in sort diagram construction; used in (sortTable.cc:273)  |
| `uniqueSortIndex = -1` | -1 | Fast symbol handling when no unique sort exists; used in (symbol.cc:239)  |
| `uniqueSortIndex = 0` | 0 | Slow symbol handling when no unique sort exists; used in (symbol.cc:239)  |

## Key Usage Patterns

**Safe Usage**: Values like `KIND`/`ERROR_SORT` (0) and `FIRST_USER_SORT` (1) are safely used in array indexing operations since they're non-negative.

**Sentinel Usage**: Negative values (`SORT_UNKNOWN`, `IMPOSSIBLE`) serve as sentinels and are explicitly checked against before array operations, as shown in the assertions (symbol.cc:496).

**Display Context**: Special handling for KIND sorts in display operations (visibleModule.cc:552-553 and visibleModule.cc:884-885).

**Notes**

The overlap between `KIND` and `ERROR_SORT` both being 0 is intentional - they represent the same concept of the least informative sort in a connected component. The system carefully prevents negative values from being used in array indexing through explicit assertions and validation.

However, Maude uses sort indices _as indices_ into arrays, for example 
`sortDiagram[position + sortIndex]` in `SortTable::traverse`
(similarly in `SortTable::ctorTraverse` and elsewhere).

*/

use std::{
  cmp::Ordering,
  fmt::Display,
  ops::{Add, SubAssign}
};
use mod2_abs::optimizable_int::OptU16;


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct SortIndex(Option<OptU16>);

impl SortIndex {
  pub const KIND                 : SortIndex = SortIndex(Some(OptU16::ZERO));
  pub const ERROR                : SortIndex = SortIndex(Some(OptU16::ZERO));
  pub const FIRST_USER_SORT      : SortIndex = SortIndex(Some(OptU16::new_unchecked(1)));
  pub const UNKNOWN              : SortIndex = SortIndex(None);
  pub const UNDEFINED            : SortIndex = SortIndex(None);
  pub const UNINITIALIZED        : SortIndex = SortIndex(Some(OptU16::ZERO));
  pub const IMPOSSIBLE           : SortIndex = SortIndex(None);
  pub const FAST_CASE_UNIQUE_SORT: SortIndex = SortIndex(None);
  pub const SLOW_CASE_UNIQUE_SORT: SortIndex = SortIndex(Some(OptU16::ZERO));
  pub const ZERO                 : SortIndex = SortIndex(Some(OptU16::ZERO));

  /// The caller must ensure that `value != u16::MAX`, which is not representable.
  #[inline(always)]
  pub fn new(value: u16) -> Result<Self, ()> {
    Ok(Self(Some(OptU16::new(value)?)))
  }
  
  /// The caller must ensure that `value != u16::MAX`, which is not representable.
  #[inline(always)]
  pub const fn new_unchecked(value: u16) -> Self {
    Self(Some(OptU16::new_unchecked(value)))
  }

  /// The caller must ensure that `value < u16::MAX`; other values are unrepresentable.
  pub fn from_usize_unchecked(value: usize) -> Self {
    Self(Some(OptU16::new_unchecked(value as u16)))
  }

  /// Returns numeric value if it exists.
  #[inline(always)]
  pub fn get(&self) -> Option<u16> {
    self.0.map(|x| x.get())
  }

  /// Returns numeric value. Panics if there is no numeric value.
  #[inline(always)]
  pub fn get_unchecked(&self) -> u16 {
    self.0.unwrap().get()
  }

  /// Returns a `usize` of the index value.
  #[inline(always)]
  pub fn idx(&self) -> Option<usize> {
    self.get().map(|x| x as usize)
  }

  /// Returns a `usize` of the index value. Panics if there is no index value.
  #[inline(always)]
  pub fn idx_unchecked(&self) -> usize {
    self.get_unchecked() as usize
  }

  // region Value Checks
  
  /// Checks that this index represents a non-error sort, i.e. it represents a positive index.
  pub fn is_positive(&self) -> bool {
    self.get().map_or(false, |x| x > 0)
  }
  
  /// Returns `true` if `self` is `UNKNOWN`, `IMPOSSIBLE`, or `FAST_CASE_UNIQUE_SORT`.
  pub fn is_none(&self) -> bool {
    self.get().is_none()
  }
  
  /// Returns `true` if `self` can be used to index an array.
  pub fn is_index(&self) -> bool {
    self.get().is_some()
  }

  /// Returns `true` if this `SortIndex` equals `SortIndex::KIND`.
  pub fn is_kind(&self) -> bool {
    *self == SortIndex::KIND
  }

  /// Returns `true` if this `SortIndex` equals `SortIndex::ERROR`.
  pub fn is_error(&self) -> bool {
    *self == SortIndex::ERROR
  }

  /// Returns `true` if this `SortIndex` equals `SortIndex::FIRST_USER_SORT`.
  pub fn is_first_user_sort(&self) -> bool {
    *self == SortIndex::FIRST_USER_SORT
  }

  /// Returns `true` if this `SortIndex` equals `SortIndex::UNKNOWN`.
  pub fn is_unknown(&self) -> bool {
    *self == SortIndex::UNKNOWN
  }

  /// Returns `true` if this `SortIndex` equals `SortIndex::UNINITIALIZED`.
  pub fn is_uninitialized(&self) -> bool {
    *self == SortIndex::UNINITIALIZED
  }

  /// Returns `true` if this `SortIndex` equals `SortIndex::IMPOSSIBLE`.
  pub fn is_impossible(&self) -> bool {
    *self == SortIndex::IMPOSSIBLE
  }

  /// Returns `true` if this `SortIndex` equals `SortIndex::FAST_CASE_UNIQUE_SORT`.
  pub fn is_fast_case_unique_sort(&self) -> bool {
    *self == SortIndex::FAST_CASE_UNIQUE_SORT
  }

  /// Returns `true` if this `SortIndex` equals `SortIndex::SLOW_CASE_UNIQUE_SORT`.
  pub fn is_slow_case_unique_sort(&self) -> bool {
    *self == SortIndex::SLOW_CASE_UNIQUE_SORT
  }
  // endregion Value Checks
}

impl Default for SortIndex {
  fn default() -> Self {
    Self::UNKNOWN
  }
}

impl PartialOrd for SortIndex {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for SortIndex {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self.get(), other.get()) {
      (None, None)       => Ordering::Equal,
      (None, Some(_))    => Ordering::Less,
      (Some(_), None)    => Ordering::Greater,
      (Some(a), Some(b)) => a.cmp(&b),
    }
  }
}

impl TryFrom<u16> for SortIndex {
  type Error = ();

  fn try_from(value: u16) -> Result<Self, Self::Error> {
    // Attempt to create an OptU16 from `value`.
    // OptU16::new(v) returns None if `v` is too large (v == u16::MAX).
    OptU16::new(value).map(|v| SortIndex(Some(v)))
  }
}

impl TryFrom<usize> for SortIndex {
  type Error = ();

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    // First, ensure `value` fits in an `OptU16`
    if value > (u16::MAX - 1) as usize {
      return Err(());
    }
    // Infallible
    Ok(SortIndex::new_unchecked(value as u16))
  }
}

impl Display for SortIndex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.get() {
      None => write!(f, "UNKNOWN"),
      Some(value) => write!(f, "{}", value),
    }
  }
}

// Arithmetic

impl SubAssign<i32> for SortIndex {
  fn sub_assign(&mut self, rhs: i32) {
    let lhs = self.get_unchecked();
    let rhs = rhs as u16;
    let new_value = lhs.checked_sub(rhs).unwrap();
    *self = SortIndex::new_unchecked(new_value);
  }
}

impl Add<usize> for SortIndex {
  type Output = SortIndex;

  fn add(self, rhs: usize) -> Self::Output {
    assert!(!self.is_none());
    let result = self.idx_unchecked() + rhs;
    result.try_into().unwrap()
  }
}