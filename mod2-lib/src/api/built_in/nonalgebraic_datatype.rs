/*!

The `NADataType` trait provides a uniform interface to types that are used to implement built-in primitive datatypes. 
So far we just use it for:

  1. getting the right numeric type for hashing `NATerm`s
  2. implementing a total order compare function for f64
  3. finalizing an `NADagNode<T>` for those values `T` that own memory (`String`)
  4. selecting the right `EquationalTheory` variant for `DagNode` construction

*/

use std::{
  fmt::Display,
  cmp::Ordering,
  any::Any,
  hash::{Hash, Hasher}
};
use std::mem::transmute;
use ordered_float::OrderedFloat;
use mod2_abs::debug;
use mod2_abs::hash::FastHasher;

use crate::{
  api::{
    built_in::{
      Bool,
      Float,
      Integer,
      NaturalNumber,
      StringBuiltIn
    },
    dag_node::DagNode
  },
  HashType,
};
use crate::core::EquationalTheory;

/// A uniform interface to types that are used to implement built-in primitive datatypes.
pub trait NADataType: Any + Clone + Display {
  const THEORY: EquationalTheory;
  
  fn hashable_bits(&self) -> HashType;
  fn compare(&self, other: &Self) -> Ordering;
  fn finalize_dag_node(_node: &mut dyn DagNode){ /* empty default impl */}
}

impl NADataType for Bool {
  const THEORY: EquationalTheory = EquationalTheory::Bool;
  fn hashable_bits(&self) -> HashType { *self as u8 as u32 }
  fn compare(&self, other: &Self) -> Ordering { other.cmp(self) }
}

impl NADataType for Float {
  const THEORY: EquationalTheory = EquationalTheory::Float;
  fn hashable_bits(&self) -> HashType {
    let bits64 = self.to_bits();
    // XOR the upper 4 bytes with the lower 4 bytes and truncate
    ((bits64 >> 32) ^ (bits64 & (u32::MAX as u64))) as HashType
  }
  
  fn compare(&self, other: &Self) -> Ordering {
    let ordered_self = OrderedFloat::from(*self);
    let ordered_other = OrderedFloat::from(*other);
    ordered_self.cmp(&ordered_other)
  }
}

impl NADataType for Integer {
  const THEORY: EquationalTheory = EquationalTheory::Integer;
  fn hashable_bits(&self) -> HashType {
    let bits64 = self.cast_unsigned();
    // XOR the upper 4 bytes with the lower 4 bytes and truncate
    ((bits64 >> 32) ^ (bits64 & (u32::MAX as u64))) as HashType
  }
  
  fn compare(&self, other: &Self) -> Ordering { other.cmp(self) }
}

impl NADataType for NaturalNumber {
  const THEORY: EquationalTheory = EquationalTheory::NaturalNumber;
  fn hashable_bits(&self) -> HashType {
    // XOR the upper 4 bytes with the lower 4 bytes and truncate
    ((self >> 32) ^ (self & (u32::MAX as u64))) as HashType
  }
  
  fn compare(&self, other: &Self) -> Ordering { self.cmp(&other) }
}

impl NADataType for StringBuiltIn {
  const THEORY: EquationalTheory = EquationalTheory::String;
  fn hashable_bits(&self) -> HashType {
    let mut hasher = FastHasher::default();
    self.hash(&mut hasher);
    let bits64 = hasher.finish();

    // XOR the upper 4 bytes with the lower 4 bytes and truncate
    ((bits64 >> 32) ^ (bits64 & (u32::MAX as u64))) as HashType
  }
  
  fn compare(&self, other: &Self) -> Ordering { self.cmp(&other) }

  fn finalize_dag_node(node: &mut dyn DagNode) {
    #[cfg(feature = "gc_debug")]
    debug!(5, "Finalizing StringDagNode");
    // Reconstitute the string so its destructor can be called.
    let (ptr, length, capacity): (*mut u8, usize, usize) = unsafe{ transmute(node.core().inline) };
    let droppable_string: StringBuiltIn = unsafe { StringBuiltIn::from_raw_parts(ptr, length, capacity) };
    drop(droppable_string)
  }
}

