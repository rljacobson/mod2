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
  mem::transmute,
  marker::PhantomData
};
use mod2_abs::{
  as_bytes,
  IString,
  hash::hash2
};
use crate::{
  core::{
    dag_node_core::{
      DagNodeFlags,
      DagNodeCore,
      DagNodeFlag,
      ThinDagNodePtr
    },
    EquationalTheory,
    gc::allocate_dag_node,
    HashConsSet
  },
  api::{
    built_in::{
      NaturalNumber,
      NADataType,
      Float,
      Integer,
      Bool,
      StringBuiltIn,
      get_built_in_symbol
    },
    Arity,
    dag_node::{
      DagNodeVectorRefMut,
      DagNodeVector,
      DagNode,
      DagNodePtr,
      arg_to_dag_node,
      arg_to_node_vec
    },
    symbol::SymbolPtr
  },
  HashType,
};

pub type BoolDagNode    = NADagNode<Bool>;
pub type FloatDagNode   = NADagNode<Float>;
pub type IntegerDagNode = NADagNode<Integer>;
pub type StringDagNode  = NADagNode<String>;
pub type NaturalNumberDagNode = NADagNode<NaturalNumber>;

#[repr(transparent)]
pub struct NADagNode<T: NADataType>(DagNodeCore, PhantomData<T>);
impl<T: NADataType> NADagNode<T> {
  pub fn value(&self) -> T {
    // Specialized because not all values are copy, and some might be ref counted.
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
          // The unwrap is guaranteed to succeed by construction.
          node.as_any_mut().downcast_mut::<Self>().unwrap().set_value(value);

          node
        }

        fn set_value(&mut self, value: $natype) {
          let ptr = self.core_mut().inline.as_mut_ptr() as *mut $natype;
          unsafe{ std::ptr::write_unaligned(ptr, value); }
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
    // The unwrap is guaranteed to succeed by construction.
    node.as_any_mut().downcast_mut::<Self>().unwrap().set_value(value);

    node
  }

  fn set_value(self: &mut Self, value: Bool) {
    let ptr = self.core_mut().inline.as_mut_ptr() as *mut Bool;
    unsafe{
      std::ptr::write_unaligned(ptr, value);
    }
  }
}

// Strings own their own memory and so need special treatment.
impl NADagNode<StringBuiltIn> {
  pub fn new(value: StringBuiltIn) -> DagNodePtr {
    let symbol   = unsafe{ get_built_in_symbol("String").unwrap_unchecked() };
    let mut node = DagNodeCore::with_theory(symbol, EquationalTheory::String);

    // Needs destruction to drop the `String` in `DagNodeCode::inline`.
    node.set_flags(DagNodeFlag::NeedsDestruction.into());
    // The unwrap is guaranteed to succeed by construction.
    node.as_any_mut().downcast_mut::<Self>().unwrap().set_value(value);

    node
  }

  fn set_value(self: &mut Self, value: StringBuiltIn) {
    let ptr = self.core_mut().inline.as_mut_ptr() as *mut StringBuiltIn;
    unsafe{
      std::ptr::write_unaligned(ptr, value);
    }
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

  fn make_canonical(&self, _hash_cons_set: &mut HashConsSet) -> DagNodePtr {
    self.as_ptr()
  }

  fn make_canonical_copy(&self, _hash_cons_set: &mut HashConsSet) -> DagNodePtr {
    self.make_clone()
  }

  fn make_clone(&self) -> DagNodePtr {
    let mut new_node = T::make_dag_node(self.value());

    // Copy over just the rewriting flags
    let rewrite_flags = self.flags() & DagNodeFlag::RewritingFlags;
    new_node.set_flags(rewrite_flags);
    new_node.set_sort_index(self.sort_index());

    new_node
  }

  // Only needed for `NADataType` that needs to release resources, which is only `StringBuiltIn`
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
