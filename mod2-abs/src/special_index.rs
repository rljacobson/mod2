/*!

A `SpecialIndex<N, E, const Reserved: u8>`, where `N` is an unsigned integer type and `E` is an enum that implements
`From<N>` and `Into<N>`, acts like the integer type `N` but can encode the variants of `E` in the largest values
representable by `N` (`N::MAX`, `N::MAX-1`, ...).

The idea is that `SpecialIndex<N, E, const Reserved: u8>` can represent `N::MAX - Reserved + 1` values, where `Reserved`
is the number of variants in `E`. (A regular `N` can represent `N::MAX + 1` values, counting zero.) A value `e: E`
should convert to some number in `0..Reserved`. Internally, `e` is mapped to `N::MAX - e.into()`.

This type is an alternative to the common practice of using a signed integer type and encoding the variants of `U` as
negative numbers. Using a signed int has the advantage that decoding to an unsigned value is a noop, but has the
disadvantage that a full half of the representable values of the unsigned `N` are no longer representable.

*/

use std::{
  marker::PhantomData,
  fmt::{Debug, Display, Formatter},
  cmp::Ordering,
  ops::{Add, AddAssign, Sub, SubAssign}
};
use num_traits::{Bounded, ConstOne, ConstZero, Unsigned};

pub trait InnerIndexType: Unsigned + Bounded + PartialOrd + Ord + PartialEq + Eq + From<u8> + Copy + Into<u64> + ConstZero + ConstOne {}
pub trait OuterEnumType<N: InnerIndexType>: TryFrom<N> + Into<N> + Copy + PartialEq + Eq {}

impl<T> InnerIndexType for T where T: Unsigned + Bounded + PartialOrd + Ord + PartialEq + Eq + From<u8> + Copy + Into<u64> + ConstZero + ConstOne {}

/// Parameterized by an integral type `N` and an enum `E` that is convertible to/from `N`,
/// `SpecialIndex<N, E, RESERVED>` represents either an `N` (for most values of `N`) OR an `E`.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SpecialIndex<N, E, const RESERVED: u8>
    where E: OuterEnumType<N>,
          N: InnerIndexType,
{
  inner   : N,
  _phantom: PhantomData<E>,
}

impl<N, E, const RESERVED: u8> SpecialIndex<N, E, RESERVED>
where
    E: OuterEnumType<N>,
    N: InnerIndexType,
{
  pub fn is_positive(&self) -> bool {
    self.is_index() && self.inner > N::ZERO
  }
}

impl<N, E, const RESERVED: u8> SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType
{

  /// Unchecked creation from an index of type `N`. This is safe, but you might get
  /// an enum variant representation when you intended to get an `N` representation.
  #[inline(always)]
  pub const fn new(index: N) -> Self {
    SpecialIndex { inner: index, _phantom: PhantomData }
  }

  pub fn from_variant(variant: E) -> Self {
    SpecialIndex { inner: Self::encode_enum(variant), _phantom: PhantomData }
  }

  /// Checks if `self` represents value `variant` of type `E`.
  #[inline(always)]
  pub fn is(&self, variant: E) -> bool {
    self.inner == Self::encode_enum(variant)
  }

  #[inline(always)]
  pub fn is_index(&self) -> bool {
    self.inner < N::max_value() - RESERVED.into() + N::ONE
  }

  /// Checked access to the index value
  #[inline(always)]
  pub fn get(&self) -> Option<N> {
    if self.is_index() {
      Some(self.inner)
    } else {
      None
    }
  }

  /// Unchecked access to the index value.
  ///
  /// While safe, this method may reinterpret an enum variant as a very large index, likely resulting in an
  /// out-of-bounds array access.
  #[inline(always)]
  pub fn get_unchecked(&self) -> N {
    debug_assert!(self.is_index(), "Called `get_unchecked()` on a non-index");
    self.inner
  }

  /// Unchecked access to the index value.
  ///
  /// While safe, this method may reinterpret an enum variant as a very large index, likely resulting in an
  /// out-of-bounds array access.
  #[inline(always)]
  pub fn idx(&self) -> usize {
    debug_assert!(self.is_index(), "Called `idx()` on a non-index");
    self.inner.into() as usize
  }

  /// Converts a variant of `E` to its internal representation as an `N` value
  #[inline(always)]
  fn encode_enum(variant: E) -> N {
    N::max_value() - variant.into()
  }

  /// Checked conversion into an enum variant.
  pub fn variant(&self) -> Option<E> {
    E::try_from(N::max_value() - self.inner).ok()
  }
}

impl<N, E, const RESERVED: u8> SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType + TryFrom<usize>
{
  /// Unchecked creation from an index of type `usize`. This is unsafe, AND you might get
  /// an enum variant representation when you intended to get an `N` representation.
  #[inline(always)]
  pub fn from_usize(index: usize) -> Self {
    let result = index.try_into(); // For inference inside the following `debug_assert`
    debug_assert!(result.is_ok(), "Called `idx()` on a non-index");
    let index = result.ok().unwrap();
    SpecialIndex { inner: index, _phantom: PhantomData }
  }
}

impl<N, E, const RESERVED: u8> Display for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N> + Display,
          N: InnerIndexType + Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if let Some(variant) = self.variant() {
      Display::fmt(&variant, f)
    } else {
      Display::fmt(&self.inner, f)
    }
  }
}

impl<N, E, const RESERVED: u8> Debug for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N> + Debug,
          N: InnerIndexType + Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if let Some(variant) = self.variant() {
      Debug::fmt(&variant, f)
    } else {
      Debug::fmt(&self.inner, f)
    }
  }
}

impl<N, E, const RESERVED: u8> Ord for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N> + Ord,
          N: InnerIndexType
{
  fn cmp(&self, other: &Self) -> Ordering {
    // Every index is greater than every variant.
    match (self.variant(), other.variant()) {

      (Some(self_variant), Some(other_variant)) => {
        self_variant.cmp(&other_variant)
      }

      (Some(_), None) => Ordering::Less,

      (None, Some(_)) => Ordering::Greater,

      (None, None) => {
        self.inner.cmp(&other.inner)
      },

    }
  }
}

impl<N, E, const RESERVED: u8> PartialOrd for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N> + Ord,
          N: InnerIndexType
{
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<N, E, const RESERVED: u8> PartialEq<N> for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType
{
  fn eq(&self, other: &N) -> bool {
    if self.is_index() {
      self.inner == *other
    } else {
      false
    }
  }
}

impl<N, E, const RESERVED: u8> PartialOrd<N> for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N> + Ord,
          N: InnerIndexType,
          SpecialIndex<N, E, RESERVED>: PartialEq<N>
{
  fn partial_cmp(&self, other: &N) -> Option<Ordering> {
    if self.is_index() {
      self.inner.partial_cmp(other)
    } else {
      Some(Ordering::Less)
    }
  }
}

impl<N, E, const RESERVED: u8> Default for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType,
{
  fn default() -> Self {
    Self{
      inner: N::max_value(),
      _phantom: Default::default(),
    }
  }
}

impl<N, E, const RESERVED: u8> From<E> for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType,
{
  #[inline(always)]
  fn from(value: E) -> Self {
    let n: N  = value.into();
    let inner = N::max_value() - n;
    SpecialIndex { inner, _phantom: PhantomData }
  }
}

// impl<N, E, const RESERVED: u8> TryFrom<N> for SpecialIndex<N, E, RESERVED>
//     where E: InnerEnumType<N>,
//           N: InnerIndexType,
// {
//   type Error = ();
//
//   #[inline(always)]
//   fn try_from(value: N) -> Result<Self, Self::Error> {
//     if value > N::max_value() - RESERVED.into() {
//       Err(())
//     } else {
//       Ok(SpecialIndex { inner: value, _phantom: PhantomData })
//     }
//   }
// }


impl<N, E, const RESERVED: u8> SubAssign<Self> for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType,
{
  fn sub_assign(&mut self, rhs: Self) {
    if let (true, true) = (self.is_index(), rhs.is_index()) {
      let lhs = self.get_unchecked();
      let rhs = rhs.get_unchecked();
      self.inner = lhs - rhs
    }
  }
}

impl<N, E, const RESERVED: u8> SubAssign<N> for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType + SubAssign,
{
  fn sub_assign(&mut self, rhs: N) {
    if self.is_index() {
      self.inner -= rhs
    }
  }
}

impl<N, E, const RESERVED: u8> AddAssign for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType + AddAssign,
{
  fn add_assign(&mut self, rhs: Self) {
    if let (true, true) = (self.is_index(), rhs.is_index()) {
      let rhs = rhs.get_unchecked();
      self.inner += rhs
    }
  }
}

impl<N, E, const RESERVED: u8> AddAssign<N> for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType + AddAssign,
{

  fn add_assign(&mut self, rhs: N) {
    if self.is_index() {
      self.inner += rhs
    }
  }
}

impl<N, E, const RESERVED: u8> Add for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType,
{
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    if let (true, true) = (self.is_index(), rhs.is_index()) {
      let lhs = self.get_unchecked();
      let rhs = rhs.get_unchecked();
      Self::new(lhs + rhs)
    } else{
      self
    }
  }
}

impl<N, E, const RESERVED: u8> Add<N> for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType,
{
  type Output = Self;

  fn add(self, rhs: N) -> Self::Output {
    if self.is_index() {
      let lhs = self.get_unchecked();
      Self::new(lhs + rhs)
    } else{
      self
    }
  }
}

impl<N, E, const RESERVED: u8> Sub for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType,
{
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    if let (true, true) = (self.is_index(), rhs.is_index()) {
      let lhs = self.get_unchecked();
      let rhs = rhs.get_unchecked();
      Self::new(lhs - rhs)
    } else{
      self
    }
  }
}

impl<N, E, const RESERVED: u8> Sub<N> for SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType,
{
  type Output = Self;

  fn sub(self, rhs: N) -> Self::Output {
    if self.is_index() {
      let lhs = self.get_unchecked();
      Self::new(lhs - rhs)
    } else{
      self
    }
  }
}

// The following is a catalog of special sentinel values used throughout the codebase. They
// really shouldn't be here, because they are (aliases for) variants of the `OuterEnumType`,
// but it's convenient to have them here nonetheless. Maybe someday I'll revisit this.

// impl<E, const RESERVED: u8> SpecialIndex<u16, E, RESERVED>
//     where E: TryFrom<u16> + Into<u16> + Copy
// {
//   #![allow(non_upper_case_globals)]
//   pub const Kind              : SpecialIndex<u16, E, RESERVED> = SpecialIndex::new(0);
//   pub const Error             : SpecialIndex<u16, E, RESERVED> = SpecialIndex::new(0);
//   pub const FirstUserSort     : SpecialIndex<u16, E, RESERVED> = SpecialIndex::new(1);
//   pub const Unknown           : SpecialIndex<u16, E, RESERVED> = SpecialIndex::new(u16::MAX); // Hack for const
//   pub const Undefined         : SpecialIndex<u16, E, RESERVED> = Self::Unknown;
//   pub const Uninitialized     : SpecialIndex<u16, E, RESERVED> = SpecialIndex::new(0);
//   pub const Impossible        : SpecialIndex<u16, E, RESERVED> = Self::Unknown;
//   pub const FastCaseUniqueSort: SpecialIndex<u16, E, RESERVED> = Self::Unknown;
//   pub const SlowCaseUniqueSort: SpecialIndex<u16, E, RESERVED> = SpecialIndex::new(0);
//   pub const Zero              : SpecialIndex<u16, E, RESERVED> = SpecialIndex::new(0);
// }

impl<N, E, const RESERVED: u8> SpecialIndex<N, E, RESERVED>
    where E: OuterEnumType<N>,
          N: InnerIndexType,
{
  #![allow(non_upper_case_globals)]
  /// Top sort in a connected component; used in `Sort` and for kind-level declarations in `SortTable`
  pub const Kind              : SpecialIndex<N, E, RESERVED> = SpecialIndex::new(N::ZERO);
  /// Error sort (same as `KIND`); used as starting point in sort computation and error checking (`SortTable`)
  pub const Error             : SpecialIndex<N, E, RESERVED> = SpecialIndex::new(N::ZERO);
  /// Starting index for user-defined sorts; used in `Sort`
  pub const FirstUserSort     : SpecialIndex<N, E, RESERVED> = SpecialIndex::new(N::ONE);
  // /// Indicates uncomputed/unknown sort; used in runtime sort checking, memory allocation, and assertions before traverse operations (`Symbol`)
  // pub const Unknown           : SpecialIndex<N, E, RESERVED> = SpecialIndex::new(N::max_value()); // Hack for const
  // /// Used by the graph coloring algorithm
  // pub const Undefined         : SpecialIndex<N, E, RESERVED> = Self::Unknown;
  /// Local variable in sort diagram construction; used in `SortTable`
  pub const Uninitialized     : SpecialIndex<N, E, RESERVED> = SpecialIndex::new(N::ZERO);
  // /// Local variable indicating impossible state in sort diagram construction; used in `SortTable`
  // pub const Impossible        : SpecialIndex<N, E, RESERVED> = Self::Unknown;
  // /// Fast symbol handling when no unique sort exists; used in `Symbol`
  // pub const FastCaseUniqueSort: SpecialIndex<N, E, RESERVED> = Self::Unknown;
  /// Slow symbol handling when no unique sort exists; used in `Symbol`
  pub const SlowCaseUniqueSort: SpecialIndex<N, E, RESERVED> = SpecialIndex::new(N::ZERO);
  pub const Zero              : SpecialIndex<N, E, RESERVED> = SpecialIndex::new(N::ZERO);
}

macro_rules! implement_constants {
    ($ty:ty) => {
        impl<E, const RESERVED: u8> SpecialIndex<$ty, E, RESERVED>
        where
            E: OuterEnumType<$ty>,
        {
            #![allow(non_upper_case_globals)]
            /// Indicates uncomputed/unknown sort; used in runtime sort checking, memory allocation, and assertions before traverse operations (`Symbol`
            pub const Unknown           : Self = Self::new(<$ty>::MAX);
            pub const None              : Self = Self::Unknown;
            /// Used by the graph coloring algorithm
            pub const Undefined         : Self = Self::Unknown;
            /// Local variable indicating impossible state in sort diagram construction; used in `SortTable`
            pub const Impossible        : Self = Self::Unknown;
            /// Fast symbol handling when no unique sort exists; used in `Symbol`
            pub const FastCaseUniqueSort: Self = Self::Unknown;
        }
    };
}

implement_constants!(u16);
implement_constants!(u32);

