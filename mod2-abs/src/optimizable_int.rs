/*!

An integer type for which `Option<T>` is the same size as `<T>`. This is a convenience wrapper around
`std::num::NonZero<T>` which allows for zero values by subtracting (adding) 1 from (to) the stored
value.

*/

use std::num::{NonZero, TryFromIntError, ZeroablePrimitive};
use num_traits::{CheckedAdd, ConstOne};

pub type OptU8    = OptInt<u8>;
pub type OptU16   = OptInt<u16>;
pub type OptU32   = OptInt<u32>;
pub type OptU64   = OptInt<u64>;
pub type OptUsize = OptInt<usize>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct OptInt<T>(NonZero<T>)
    where T: ZeroablePrimitive + CheckedAdd + ConstOne + std::ops::Sub<Output = T>;

impl<T> OptInt<T>
    where T: ZeroablePrimitive + CheckedAdd + ConstOne + std::ops::Sub<Output = T>
{
  pub const ZERO: OptInt<T> = OptInt(unsafe{ NonZero::new_unchecked(T::ONE) });

  pub fn new(value: T) -> Result<OptInt<T>, ()> {
    value.checked_add(&T::ONE)
         .map(|v| unsafe { OptInt(NonZero::new_unchecked(v)) })
         .ok_or(())
  }

  pub fn get(self) -> T {
    // Since `stored = original + 1 > 0`, this cannot underflow
    self.0.get() - T::ONE
  }
}

macro_rules! impl_into_optint {
    ($ty:ty) => {
        impl OptInt<$ty> {
            /// The caller must ensure that `value != MAX`, which is not representable.
            pub const fn new_unchecked(value: $ty) -> OptInt<$ty> {
                OptInt(unsafe{ NonZero::new_unchecked(value + <$ty>::ONE) })
            }
            /// Despite the name, this method is infallible.
            pub const fn get_unchecked(&self) -> $ty {
                self.0.get() - <$ty>::ONE
            }
        }
        // Rust is too stupid to implement `Into<T> for OptInt<T>` when `T` is generic, so we have to do this.
        impl Into<$ty> for OptInt<$ty> {
            fn into(self) -> $ty {
                self.get()
            }
        }
    };
}

impl_into_optint!(u8);
impl_into_optint!(u16);
impl_into_optint!(u32);
impl_into_optint!(u64);
impl_into_optint!(usize);
