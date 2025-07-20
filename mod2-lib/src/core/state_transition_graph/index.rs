// ToDo: Reduce the boilerplate for this and `core::index`.

use std::fmt::{Display, Formatter};
use mod2_abs::special_index::{OuterEnumType, SpecialIndex};

pub type PositionIndex    = SpecialIndex<RawPositionIndex, PositionStateSentinel, 2>;
pub type RawPositionIndex = u32;
pub type PositionDepth    = SpecialIndex<RawPositionDepth, PositionStateDepthSentinel, 2>;
pub type RawPositionDepth = u32;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum PositionStateSentinel {
  None      = 0, // In Maude: called both DEFAULT and UNDEFINED and has value -1
  Unbounded = 1, // In Maude: Max i32 value
}

// region impls for PositionStateSentinel

impl Display for PositionStateSentinel {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      PositionStateSentinel::None => {
        write!(f, "None")
      }
      PositionStateSentinel::Unbounded => {
        write!(f, "Unbounded")
      }
    }
  }
}

impl TryFrom<u8> for PositionStateSentinel {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(PositionStateSentinel::None),
      1 => Ok(PositionStateSentinel::Unbounded),
      _ => Err(()),
    }
  }
}

impl TryFrom<RawPositionIndex> for PositionStateSentinel {
type Error = ();

fn try_from(value: RawPositionIndex) -> Result<Self, Self::Error> {
  // Delegate to the u8 version if within range
  u8::try_from(value).ok().and_then(|v| v.try_into().ok()).ok_or(())
}
}

impl From<PositionStateSentinel> for RawPositionIndex {
fn from(value: PositionStateSentinel) -> Self {
  value as u8 as RawPositionIndex
}
}

impl OuterEnumType<RawPositionIndex> for PositionStateSentinel {}

// endregion impls for PositionStateSentinel


#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum PositionStateDepthSentinel {
  // All the way down to the leaf nodes, with extension
  Unbounded           = 0, // In Maude: Max i32 value
  // At top, no extension
  TopWithoutExtension = 1,
  // At top, with extension
  TopWithExtension    = 2,
}

// region impls for PositionStateDepthSentinel

impl Display for PositionStateDepthSentinel {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      PositionStateDepthSentinel::Unbounded => {
        write!(f, "Unbounded")
      }

      PositionStateDepthSentinel::TopWithoutExtension => {
        write!(f, "TopWithoutExtension")
      }

      PositionStateDepthSentinel::TopWithExtension => {
        write!(f, "TopWithExtension")
      }
    }
  }
}


impl TryFrom<u8> for PositionStateDepthSentinel {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(PositionStateDepthSentinel::Unbounded),
      1 => Ok(PositionStateDepthSentinel::TopWithoutExtension),
      2 => Ok(PositionStateDepthSentinel::TopWithExtension),
      _ => Err(()),
    }
  }
}

impl TryFrom<RawPositionDepth> for PositionStateDepthSentinel {
  type Error = ();

  fn try_from(value: RawPositionDepth) -> Result<Self, Self::Error> {
    // Delegate to the u8 version if within range
    u8::try_from(value).ok().and_then(|v| v.try_into().ok()).ok_or(())
  }
}

impl From<PositionStateDepthSentinel> for RawPositionDepth {
  fn from(value: PositionStateDepthSentinel) -> Self {
    value as u8 as RawPositionDepth
  }
}

impl OuterEnumType<RawPositionDepth> for PositionStateDepthSentinel {}

// endregion impls for PositionStateDepthSentinel
