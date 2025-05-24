/*!

`NADagNode`

# Memory

```ignore
DagNodeCore::args: null
DagNodeCore::inline: NAValueType
```

`NADagNode<BuiltInString>` needs destruction, as a `String` owns allocated memory.


*/

use std::{
  any::Any,
  cmp::{
    max,
    Ordering
  },
  ops::DerefMut,
  mem::transmute
};
use std::marker::PhantomData;
use mod2_abs::{as_bytes, IString};
use mod2_abs::hash::hash2;
use crate::{core::{
  dag_node_core::{
    DagNodeCore,
    DagNodeFlags,
    DagNodeFlag,
    ThinDagNodePtr
  },
  EquationalTheory,
  gc::{
    allocate_dag_node,
    increment_active_node_count
  },
}, api::{
  Arity,
  built_in::{
    NADataType,
    Float,
    Integer,
    NaturalNumber,
    Bool,
    StringBuiltIn,
    get_built_in_symbol,
  },
  dag_node::{
    DagNodeVector,
    DagNodeVectorRefMut,
    DagNode,
    DagNodePtr,
    arg_to_dag_node,
    arg_to_node_vec
  },
  symbol::SymbolPtr,
}, HashType};

pub type BoolDagNode    = NADagNode<Bool>;
pub type FloatDagNode   = NADagNode<Float>;
pub type IntegerDagNode = NADagNode<Integer>;
pub type StringDagNode  = NADagNode<String>;
pub type NaturalNumberDagNode = NADagNode<NaturalNumber>;

pub struct NADagNode<T: NADataType>(DagNodeCore, PhantomData<T>);
impl<T: NADataType> NADagNode<T> {
  pub fn value(&self) -> T {
    T::value_from_dag_node(self)
  }
}

/// Implementation for `NADataType` that implement `Copy`
macro_rules! impl_na_dag_node {
    ($natype:ty) => {
      impl NADagNode<$natype> {
        pub fn new(value: $natype) -> DagNodePtr {
          let symbol   = unsafe{ get_built_in_symbol(stringify!($natype)).unwrap_unchecked() };
          let mut node = DagNodeCore::with_theory(symbol, <$natype as NADataType>::THEORY);

          node.core_mut().inline[..(size_of::<$natype>())].copy_from_slice(as_bytes(&value));
          node
        }
      }
    };
}

// impl_na_dag_node!(Bool);
impl_na_dag_node!(Float);
impl_na_dag_node!(Integer);
impl_na_dag_node!(NaturalNumber);

// Bool has a symbol for each value, so its constructor is special
impl NADagNode<Bool> {
  pub fn new(value: Bool) -> DagNodePtr {
    let symbol = if value {
      unsafe { get_built_in_symbol("true").unwrap_unchecked() }
    } else {
      unsafe { get_built_in_symbol("false").unwrap_unchecked() }
    };
    let mut node = DagNodeCore::with_theory(symbol, <Bool as NADataType>::THEORY);

    node.core_mut().inline[..(size_of::<Bool>())].copy_from_slice(as_bytes(&value));
    node
  }
}

// Strings own their own memory and so need special treatment.
impl NADagNode<StringBuiltIn> {
  pub fn new(value: StringBuiltIn) -> DagNodePtr {
    let symbol   = unsafe{ get_built_in_symbol("String").unwrap_unchecked() };
    let mut node = DagNodeCore::with_theory(symbol, EquationalTheory::String);

    // Needs destruction to drop the `String` in `DagNodeCode::inline`.
    node.set_flags(DagNodeFlag::NeedsDestruction.into());

    // let (ptr, length, capacity) = value.into_raw_parts();
    node.core_mut().inline =  unsafe{ transmute(value.into_raw_parts()) };
    node
  }
}


impl<T: NADataType> DagNode for NADagNode<T> {
  #[inline(always)]
  fn as_any(&self) -> &dyn Any {
    self
  }

  #[inline(always)]
  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  #[inline(always)]
  fn as_ptr(&self) -> DagNodePtr {
    DagNodePtr::new(self as *const dyn DagNode as *mut dyn DagNode)
  }

  fn structural_hash(&self) -> HashType {
    hash2(self.symbol().hash(), self.value().hashable_bits())
  }

  #[inline(always)]
  fn core(&self) -> &DagNodeCore {
    &self.0
  }

  #[inline(always)]
  fn core_mut(&mut self) -> &mut DagNodeCore {
    &mut self.0
  }

  #[inline(always)]
  fn iter_args(&self) -> Box<dyn Iterator<Item=DagNodePtr>> {
    Box::new(std::iter::empty::<DagNodePtr>())
  }

  // Only called for `NADataType` that needs to release resources, which is only `StringBuiltIn`
  #[inline(always)]
  fn finalize(&mut self) {
    T::finalize_dag_node(self);
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_bool_dag_node() {
    let value = true;
    let node_ptr = BoolDagNode::new(value);
    let maybe_node: Option<&BoolDagNode> = node_ptr.as_any().downcast_ref();
    assert!(maybe_node.is_some());

    let node = maybe_node.unwrap();
    let value_from_node = node.value();
    println!("DagNode {} has value {}", node_ptr, value_from_node);

    assert_eq!(value_from_node, value);

    let value = false;
    let node_ptr = BoolDagNode::new(value);
    let maybe_node: Option<&BoolDagNode> = node_ptr.as_any().downcast_ref();
    assert!(maybe_node.is_some());

    let node = maybe_node.unwrap();
    let value_from_node = node.value();
    println!("DagNode {} has value {}", node_ptr, value_from_node);

    assert_eq!(value_from_node, value);
  }

  #[test]
  fn create_float_dag_node() {
    let value = std::f64::consts::PI;
    let node_ptr = FloatDagNode::new(value);
    let maybe_node: Option<&FloatDagNode> = node_ptr.as_any().downcast_ref();
    assert!(maybe_node.is_some());

    let node = maybe_node.unwrap();
    let value_from_node = node.value();
    println!("DagNode {} has value {}", node_ptr, value_from_node);

    assert_eq!(value_from_node, value);
  }

  #[test]
  fn create_integer_dag_node() {
    let value = -1729;
    let node_ptr = IntegerDagNode::new(value);
    let maybe_node: Option<&IntegerDagNode> = node_ptr.as_any().downcast_ref();
    assert!(maybe_node.is_some());

    let node = maybe_node.unwrap();
    let value_from_node = node.value();
    println!("DagNode {} has value {}", node_ptr, value_from_node);

    assert_eq!(value_from_node, value);
  }

  #[test]
  fn create_natural_dag_node() {
    let value = 1981;
    let node_ptr = NaturalNumberDagNode::new(value);
    let maybe_node: Option<&NaturalNumberDagNode> = node_ptr.as_any().downcast_ref();
    assert!(maybe_node.is_some());

    let node = maybe_node.unwrap();
    let value_from_node = node.value();
    println!("DagNode {} has value {}", node_ptr, value_from_node);

    assert_eq!(value_from_node, value);
  }
  
  #[test]
  fn create_string_dag_node() {
    let value = String::from("Hello, world!");
    let node_ptr = StringDagNode::new(value.clone());
    let maybe_node: Option<&StringDagNode> = node_ptr.as_any().downcast_ref();
    assert!(maybe_node.is_some());

    let node = maybe_node.unwrap();
    let value_from_node = node.value();
    println!("DagNode {} has value {}", node_ptr, value_from_node);

    assert_eq!(value_from_node, value);
  }
}