/*!

A vector allocated from Bucket storage.

*/

use std::{
  ops::{Index, IndexMut},
  marker::PhantomPinned,
  cmp::min
};

use crate::{
  core::{
    allocator::acquire_storage_allocator
  }
};
use crate::api::dag_node::DagNodeVector;

pub type GCVectorRefMut<T> = &'static mut GCVector<T>;

// ToDo: Use a generic sized parameter `T` instead of a `DagNodePtr`.
pub struct GCVector<T: 'static> {
  length  : usize,
  capacity: usize,
  data    : &'static mut [T],

  // Opt out of `Unpin`
  _pin    : PhantomPinned,
}

impl<T: Copy + 'static> GCVector<T> {

  // region Constructors

  /// Creates a new empty vector with the given capacity.
  pub fn with_capacity(capacity: usize) -> GCVectorRefMut<T> {
    unsafe {
      let node_vector_ptr: *mut GCVector<T> =
          { acquire_storage_allocator().allocate_storage(size_of::<GCVector<T>>()) as *mut GCVector<T> };
      let node_vector: &mut GCVector<T>     = node_vector_ptr.as_mut_unchecked();

      // Initialize the NodeVector
      node_vector.length   = 0;
      node_vector.capacity = capacity;

      // Allocate the memory slice. Two separate allocations are needed to maintain alignment.
      let needed_memory    = capacity * size_of::<T>();
      let data_ptr         = { acquire_storage_allocator().allocate_storage(needed_memory) as *mut T };
      node_vector.data     = std::slice::from_raw_parts_mut(data_ptr, capacity);

      node_vector
    }
  }

  /// Creates a new `NodeVector` from the given slice. The capacity of the
  /// new `NodeVector` is equal to its length.
  pub fn from_slice(vec: &[T]) -> GCVectorRefMut<T> {
    let capacity = vec.len();

    let node_vector_mut: GCVectorRefMut<T> = GCVector::with_capacity(capacity);

    // Copy contents of vec into node_vector.data
    for (i, &item) in vec.iter().enumerate() {
      node_vector_mut.data[i] = item;
    }

    node_vector_mut.length = capacity;

    node_vector_mut
  }

  /// Creates an identical shallow copy, allocating new memory. The copy
  /// has the same capacity as the original.
  pub fn copy(&self) -> GCVectorRefMut<T> {
    GCVector::copy_with_capacity(self, self.capacity)
  }

  /// Makes a copy of this node but with `new_capacity`. If `self.length` > `new_capacity`,
  /// nodes are truncated.
  pub fn copy_with_capacity(&self, new_capacity: usize) -> GCVectorRefMut<T> {
    if new_capacity > self.capacity {
      let new_vector_mut: GCVectorRefMut<T> = GCVector::with_capacity(new_capacity);

      new_vector_mut.length = self.length;

      for i in 0..self.length {
        new_vector_mut.data[i] = self.data[i];
      }

      new_vector_mut
    }
    else {
      // To keep things simple, we copy everything up to `new_capacity` even if
      // `length` is shorter.
      let new_vector = GCVector::from_slice(&self.data[0..new_capacity]);

      new_vector.length = min(self.length, new_capacity);

      new_vector
    }
  }

  // endregion Constructors

  // Immutable iterator
  pub fn iter(&'static self) -> std::slice::Iter<'static, T> {
    self.data[..self.length].iter()
  }

  // Mutable iterator
  pub fn iter_mut(&'static mut self) -> std::slice::IterMut<'static, T> {
    self.data[..self.length].iter_mut()
  }

  pub fn len(&self) -> usize {
    self.length
  }

  pub fn capacity(&self) -> usize {
    self.capacity
  }

  pub fn is_empty(&self) -> bool { self.len() == 0 }

  /// Pushes the given node onto the (end) of the vector if there is enough capacity.
  pub fn push(&mut self, node: T) {

    #[cfg(feature = "gc_debug")]
    // Catches bugs in GC allocator
    if self.length > self.capacity
        || self.data.len() != self.capacity
    {
      panic!("node_vec.len: {}, capacity: {}, data.len: {}", self.length, self.capacity, self.data.len());
    }

    if self.length == self.capacity {
      panic!("node_vec.len: {}, capacity: {}, data.len: {}", self.length, self.capacity, self.data.len());
      // ToDo: Should the vector grow geometrically?
      // let new_vec = self.copy_with_capacity(self.capacity + 1);
      // std::mem::swap(self, new_vec);
    }

    self.data[self.length] = node;
    self.length += 1;
  }

  pub fn pop(&mut self) -> Option<T> {
    if self.length == 0 {
      return None;
    }

    self.length -= 1;

    Some(self.data[self.length])
  }
}

impl<T> Index<usize> for GCVector<T> {
  type Output = T;

  fn index(&self, index: usize) -> &Self::Output {
    assert!(index < self.length);
    &self.data[index]
  }
}

impl<T> IndexMut<usize> for GCVector<T> {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    assert!(index < self.length);
    &mut self.data[index]
  }
}

impl<'a, T> IntoIterator for &'a GCVector<T> {
  type Item = &'a T;
  type IntoIter = std::slice::Iter<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    self.data.iter()
  }
}

impl<'a, T> IntoIterator for &'a mut GCVector<T> {
  type Item = &'a mut T;
  type IntoIter = std::slice::IterMut<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    self.data.iter_mut()
  }
}
