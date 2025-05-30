/*!

The `NADataType` trait provides a uniform interface to types that are used to implement built-in primitive datatypes.
It holds the type-specific implementation for `NATerm`s and `NADagNode`s.

*/

use std::{
  any::Any,
  cmp::Ordering,
  fmt::Display,
  hash::{Hash, Hasher},
  mem::transmute,
};

use ordered_float::OrderedFloat;

use mod2_abs::{
  debug,
  hash::FastHasher
};

use crate::{
  api::{
    built_in::{
      Bool,
      BoolDagNode,
      Float,
      Integer,
      NaturalNumber,
      StringBuiltIn,
      NADagNode,
    },
    dag_node::{DagNode, DagNodePtr}
  },
  core::EquationalTheory,
  HashType,
};

/// A uniform interface to types that are used to implement built-in primitive datatypes.
pub trait NADataType: Any + Clone + Display + Sized {
  const THEORY: EquationalTheory;

  fn hashable_bits(&self) -> HashType;
  fn compare(&self, other: &Self) -> Ordering;
  fn finalize_dag_node(_node: &mut dyn DagNode){ /* empty default impl */}

  // Default impl for types that are `Copy`
  fn value_from_dag_node(node: &NADagNode<Self>) -> Self {
    // Reconstitute the value from its raw bytes
    let ptr = node.core().inline.as_ptr() as *const Self;
    unsafe {
      std::ptr::read_unaligned(ptr)
    }
  }

  fn make_dag_node(value: Self) -> DagNodePtr;
}

impl NADataType for Bool {
  const THEORY: EquationalTheory = EquationalTheory::Bool;
  fn hashable_bits(&self) -> HashType { *self as u8 as u32 }
  fn compare(&self, other: &Self) -> Ordering { other.cmp(self) }
  fn make_dag_node(value: Self) -> DagNodePtr{ NADagNode::<Self>::new(value) }
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
  fn make_dag_node(value: Self) -> DagNodePtr{ NADagNode::<Self>::new(value) }
}

impl NADataType for Integer {
  const THEORY: EquationalTheory = EquationalTheory::Integer;
  fn hashable_bits(&self) -> HashType {
    let bits64 = self.cast_unsigned();
    // XOR the upper 4 bytes with the lower 4 bytes and truncate
    ((bits64 >> 32) ^ (bits64 & (u32::MAX as u64))) as HashType
  }

  fn compare(&self, other: &Self) -> Ordering { other.cmp(self) }
  fn make_dag_node(value: Self) -> DagNodePtr{ NADagNode::<Self>::new(value) }
}

impl NADataType for NaturalNumber {
  const THEORY: EquationalTheory = EquationalTheory::NaturalNumber;
  fn hashable_bits(&self) -> HashType {
    // XOR the upper 4 bytes with the lower 4 bytes and truncate
    ((self >> 32) ^ (self & (u32::MAX as u64))) as HashType
  }

  fn compare(&self, other: &Self) -> Ordering { self.cmp(&other) }
  fn make_dag_node(value: Self) -> DagNodePtr{ NADagNode::<Self>::new(value) }
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
    let ptr = node.core().inline.as_ptr() as *const Self;
    let droppable_string = unsafe { std::ptr::read_unaligned(ptr) };

    drop(droppable_string)
  }

  fn value_from_dag_node(node: &NADagNode<Self>) -> StringBuiltIn {
    // Reconstitute the string so its destructor can be called.
    let ptr = node.core().inline.as_ptr() as *const Self;
    let cloneable_string = unsafe { std::ptr::read_unaligned(ptr) };

    let value = cloneable_string.clone();
    // Do not call destructor on `cloneable_string`.
    std::mem::forget(cloneable_string);

    value
  }
  fn make_dag_node(value: Self) -> DagNodePtr{ NADagNode::<Self>::new(value) }
}

