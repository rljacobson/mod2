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

use crate::{
  core::{
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
  },
  api::{
    Arity,
    built_in::{
      NADataType,
      Float,
      Integer,
      NaturalNumber,
      Bool,
      StringBuiltIn
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
  }
};
use crate::api::built_in::get_built_in_symbol;

pub type BoolDagNode    = NADagNode<Bool>;
pub type FloatDagNode   = NADagNode<Float>;
pub type IntegerDagNode = NADagNode<Integer>;
pub type StringDagNode  = NADagNode<String>;
pub type NaturalNumberDagNode = NADagNode<NaturalNumber>;

pub struct NADagNode<T: NADataType>(DagNodeCore, PhantomData<T>);

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

        pub fn value(&self) -> $natype {
          // Reconstitute the value from its raw bytes
          let slice = &self.core().inline;
          let value: $natype = unsafe {
            std::ptr::read_unaligned(slice.as_ptr() as *const $natype)
          };
            value
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
      unsafe{ get_built_in_symbol("true").unwrap_unchecked() }
    } else {
      unsafe{ get_built_in_symbol("false").unwrap_unchecked() }
    };
    let mut node = DagNodeCore::with_theory(symbol, <Bool as NADataType>::THEORY);

    node.core_mut().inline[..(size_of::<Bool>())].copy_from_slice(as_bytes(&value));
    node
  }

  pub fn value(&self) -> Bool {
    let slice = &self.core().inline;
    let value: Bool = unsafe {
      std::ptr::read_unaligned(slice.as_ptr() as *const Bool)
    };
    value
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

  pub fn value(&self) -> StringBuiltIn {
    // Reconstitute the string so its destructor can be called.
    let (ptr, length, capacity): (*mut u8, usize, usize) = unsafe{ transmute(self.core().inline) };
    let cloneable_string: StringBuiltIn = unsafe { StringBuiltIn::from_raw_parts(ptr, length, capacity) };
    
    let value = cloneable_string.clone();
    // Do not call destructor on `cloneable_string`
    std::mem::forget(cloneable_string);
    
    value
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