///! The `DagNodeCore::args` field can either hold a single `DagNode` inline, or point to a `DagNodeVector`, or be null.
///! It is convenient to have an "expanded" representation that can be passed around independently of the containing 
///! `DagNode`.
use std::ptr::null_mut;
use crate::{
  api::{
    Arity,
    arg_to_dag_node,
    arg_to_node_vec,
    DagNodePtr,
    DagNodeVector,
    DagNodeVectorRefMut
  }
};

/// An enum holding either nothing, an inline `DagNodePtr`, or a pointer to a `DagNodeVector`.
pub enum DagNodeArguments {
  None,
  Inline(DagNodePtr),
  Vec(DagNodeVectorRefMut),
}

impl DagNodeArguments {
  /// Produces a pointer value suitable for assigning directly to the args field.
  pub fn as_args(&self) -> *mut u8 {
    match self {
      DagNodeArguments::None => { null_mut() }
      DagNodeArguments::Inline(node_ptr) => { node_ptr.as_mut_ptr() as *mut u8 }
      DagNodeArguments::Vec(node_vec) => { ((*node_vec) as *const DagNodeVector) as *mut u8 }
    }
  }

  /// Extracts the args of the provided `dag_node` as an expanded `DagNodeArguments`.
  pub fn from_node(dag_node: DagNodePtr) -> Self {
    // The empty case
    if dag_node.core().args.is_null() {
      DagNodeArguments::None
    } // The vector case
    else if dag_node.core().needs_destruction() {
      let node_vector: DagNodeVectorRefMut = arg_to_node_vec(dag_node.core().args);
      DagNodeArguments::Vec(node_vector)
    } // The singleton case
    else {
      // Guaranteed to be non-null.
      let node: DagNodePtr = arg_to_dag_node(dag_node.core().args);
      DagNodeArguments::Inline(node)
    }
  }
  
  pub fn from_args(args: *mut u8, arity: Arity) -> Self {
    if arity.get() == 0 || args.is_null() {
      DagNodeArguments::None
    } else if arity.get() == 1 && !args.is_null() {
      DagNodeArguments::Inline(arg_to_dag_node(args))
    } else if arity.get() > 1 && !args.is_null() {
      DagNodeArguments::Vec(arg_to_node_vec(args))
    } else { 
      panic!("arity is incompatible with args pointer");
    }
  }
  
  pub fn is_empty(&self) -> bool {
    if *self == DagNodeArguments::None { true } else { false }
  }
  
  pub fn is_inline(&self) -> bool {
    if let DagNodeArguments::Inline(_) = self { true } else { false }
  }
  
  pub fn is_vec(&self) -> bool {
    if let DagNodeArguments::Vec(_) = self { true } else { false }
  }
  
  pub fn len(&self) -> usize {
    match self {
      DagNodeArguments::None => { 0 }
      DagNodeArguments::Inline(_) => { 1 }
      DagNodeArguments::Vec(node_vec) => { node_vec.len() }
    }
  }
  
  pub fn iter(&self) -> ArgumentsIterator {
    self.clone().into_iter()
  }
}

impl PartialEq for DagNodeArguments {
  fn eq(&self, other: &DagNodeArguments) -> bool {
    match (self, other) {
      
      (DagNodeArguments::None, DagNodeArguments::None) => true,
      
      (DagNodeArguments::Inline(self_ptr), DagNodeArguments::Inline(other_ptr)) => { 
        self_ptr == other_ptr
      },

      (DagNodeArguments::Vec(_), DagNodeArguments::Vec(_)) => { 
        self.as_args() == other.as_args()
      },
      
      _ => false,
    }
  }
}

impl Eq for DagNodeArguments {}

impl Clone for DagNodeArguments {
  fn clone(&self) -> Self {
    let arity = Arity::new_unchecked(self.len() as u16);
    Self::from_args(self.as_args(), arity)
  }
}

impl IntoIterator for DagNodeArguments {
  type Item = DagNodePtr;
  type IntoIter = ArgumentsIterator;

  fn into_iter(self) -> Self::IntoIter {
    ArgumentsIterator{
      args: self,
      idx: 0
    }
  }
}


pub struct ArgumentsIterator {
  idx: usize,
  args: DagNodeArguments,
}

impl Iterator for ArgumentsIterator {
  type Item = DagNodePtr;

  fn next(&mut self) -> Option<Self::Item> {
    match (&self.args, self.idx) {
      
      (DagNodeArguments::Inline(node), 0) => {
        self.idx += 1;
        Some(*node)
      }
      
      (DagNodeArguments::Vec(node_vector), idx) if idx < node_vector.len() => {
        self.idx += 1;
        Some(node_vector[idx - 1])
      }
      
      _ => None,
      
    }
  }
}