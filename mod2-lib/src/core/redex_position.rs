/*!

A `RedexPosition` holds position information of a potential redex.

*/

use enumflags2::{bitflags, BitFlags};
use crate::{
  api::DagNodePtr,
  core::{ArgIndex, VariableIndex}
};

#[bitflags]
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum RedexPositionFlag {
  Stale,
  Eager,
}
// Local convenience
use RedexPositionFlag::{Eager, Stale};
pub type RedexPositionFlags = BitFlags<RedexPositionFlag>;

pub struct RedexPosition {
  // ToDo: These need to be marked!
  pub dag_node    : DagNodePtr,
  pub parent_index: VariableIndex, // These indices can be UNDEFINED/NONE
  pub arg_index   : ArgIndex,
  pub flags       : RedexPositionFlags,
}

impl RedexPosition {
  pub fn is_stale(&self) -> bool {
    self.flags.contains(Stale)
  }

  pub fn is_eager(&self) -> bool {
    self.flags.contains(Eager)
  }

  pub fn set_stale(&mut self, value: bool) {
    if value {
      self.flags.insert(Stale);
    } else {
      self.flags.remove(Stale);
    }
  }

  pub fn set_eager(&mut self, value: bool) {
    if value {
      self.flags.insert(Eager);
    } else {
      self.flags.remove(Eager);
    }
  }
}
