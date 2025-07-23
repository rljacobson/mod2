/*!

This module defines special indexing types for indexes that combines unknown/impossible and
uninitialized/error states with "valid" index states that can index into arrays.

So far this includes `SortIndex`, `SlotIndex`, and `VariableIndex`. These index
types share the same enum of sentinel/special values for the sake of simplicity.

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

use std::fmt::{Display, Formatter};
use mod2_abs::special_index::{OuterEnumType, SpecialIndex};

pub type RawSortIndex       = u16;
pub type RawRuleIndex       = u16;
pub type RawSlotIndex       = u16;
pub type RawArgIndex        = u16;
// Can probably be `u16`, but Maude uses `i32`.
pub type RawVariableIndex   = u32;
// Can probably be `u16`, but Maude uses `i32`.
pub type RawSymbolIndex     = u32;
// Maude uses `i32`, but that's probably too large, and smaller state tables are more cache friendly.
pub type RawStateGraphIndex = u16;

/// The index of a sort within its kind
pub type SortIndex       = SpecialIndex<RawSortIndex      , SentinelIndex, 2>;
/// Represents a position in the match stack
pub type SlotIndex       = SpecialIndex<RawSlotIndex      , SentinelIndex, 2>;
/// Represents a position in the argument list
pub type ArgIndex        = SpecialIndex<RawSlotIndex      , SentinelIndex, 2>;
/// The index of a variable within its parent module
pub type VariableIndex   = SpecialIndex<RawVariableIndex  , SentinelIndex, 2>;
/// The index of a symbol within its parent module
pub type SymbolIndex     = SpecialIndex<RawSymbolIndex    , SentinelIndex, 2>;
/// Internal state representation of a `StateTransitionGraph`
pub type StateGraphIndex = SpecialIndex<RawStateGraphIndex, SentinelIndex, 2>;
/// Index of a rule in the `RuleTable`
pub type RuleIndex       = SpecialIndex<RawRuleIndex      , SentinelIndex, 2>;


#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum SentinelIndex {
  None,
  /// Used in `RewritingContext`, indicates that a term does not need rebuilding.
  RootOk,
}

impl SentinelIndex {
  #![allow(non_upper_case_globals)]
  pub const Unknown   : SentinelIndex = SentinelIndex::None;
  pub const Undefined : SentinelIndex = SentinelIndex::None;
  pub const Impossible: SentinelIndex = SentinelIndex::None;
}

impl Display for SentinelIndex {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      SentinelIndex::None => {
        write!(f, "None")
      }
      SentinelIndex::RootOk => {
        write!(f, "RootOk")
      }
    }
  }
}

impl TryFrom<u8> for SentinelIndex {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(SentinelIndex::None),
      1 => Ok(SentinelIndex::RootOk),
      _ => Err(()),
    }
  }
}

macro_rules! implement_from_traits {
    ($ty:ty) => {
      impl TryFrom<$ty> for SentinelIndex {
        type Error = ();

        fn try_from(value: $ty) -> Result<Self, Self::Error> {
          // Delegate to the u8 version if within range
          u8::try_from(value).ok().and_then(|v| v.try_into().ok()).ok_or(())
        }
      }

      impl From<SentinelIndex> for $ty {
        fn from(value: SentinelIndex) -> Self {
          value as u8 as $ty
        }
      }

      impl OuterEnumType<$ty> for SentinelIndex {}

    };
}

implement_from_traits!(RawSortIndex);
// implement_from_traits!(RawSlotIndex); // Same type as `RawSortIndex`
// implement_from_traits!(RawArgIndex);  // Same type as `RawSortIndex`
implement_from_traits!(RawVariableIndex);
// implement_from_traits!(RawSymbolIndex);     // Same as `RawVariableIndex`
// implement_from_traits!(RawStateGraphIndex); // Same as `RawVariableIndex`



#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(test)]
  mod tests {
    use std::cmp::Ordering;
    use super::*;

    #[test]
    fn check_values_round_trip() {
      let a = VariableIndex::from_variant(SentinelIndex::None);
      let b = VariableIndex::from_variant(SentinelIndex::RootOk);

      assert!(a.is(SentinelIndex::None));
      assert!(b.is(SentinelIndex::RootOk));

      assert_eq!(a.variant(), Some(SentinelIndex::None));
      assert_eq!(b.variant(), Some(SentinelIndex::RootOk));

      assert!(!a.is_index());
      assert!(!b.is_index());

      let c: VariableIndex = VariableIndex::new(RawVariableIndex::MAX-2);
      assert!(c.is_index());
      assert_eq!(c.get(), Some(RawVariableIndex::MAX-2));
    }

    #[test]
    fn variants_order_by_variant_value() {
      let a = VariableIndex::from_variant(SentinelIndex::None);
      let b = VariableIndex::from_variant(SentinelIndex::RootOk);
      assert!(a < b);
      assert!(b > a);
      assert_eq!(a.cmp(&a), Ordering::Equal);
    }

    #[test]
    fn variant_is_less_than_non_variant() {
      let a = VariableIndex::from_variant(SentinelIndex::None);
      let b = VariableIndex::new(0);
      assert!(a < b);
      assert!(b > a);
    }

    #[test]
    fn non_variants_order_by_inner_value() {
      let a = VariableIndex::new(10);
      let b = VariableIndex::new(20);
      let c = VariableIndex::new(10);

      assert!(a < b);
      assert!(b > a);
      assert_eq!(a.cmp(&c), Ordering::Equal);
    }

    #[test]
    fn symmetry_and_equality() {
      let a = VariableIndex::from_variant(SentinelIndex::None);
      let b = VariableIndex::from_variant(SentinelIndex::None);
      assert_eq!(a.cmp(&b), Ordering::Equal);
      assert_eq!(b.cmp(&a), Ordering::Equal);

      let c = VariableIndex::new(5);
      let d = VariableIndex::new(5);
      assert_eq!(c >= d, true);
      assert_eq!(d <= c, true);
    }
  }
}
