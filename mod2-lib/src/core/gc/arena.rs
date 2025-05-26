/*!

An arena allocator for `DagNode`s.

*/

use std::{
  mem::MaybeUninit,
  ptr::null_mut
};

use crate::{
  core::{
    dag_node_core::DagNodeCore,
    gc::node_allocator::ARENA_SIZE
  }
};

#[repr(align(8))]
pub struct Arena {
  pub(crate) data: [DagNodeCore; ARENA_SIZE],
}

impl Arena {
  #[inline(always)]
  pub fn allocate_new_arena() -> Box<Arena> {

    // Create an uninitialized array
    let data: [MaybeUninit<DagNodeCore>; ARENA_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };

    /* Each node is initialized on allocation, so we don't bother with this.
    // Initialize each element
    for elem in &mut data {
      unsafe {
        std::ptr::write(elem.as_mut_ptr(), DagNode::default());
      }
    }
    */

    let arena = Box::new(Arena{
      data: unsafe { std::mem::transmute::<_, [DagNodeCore; ARENA_SIZE]>(data) }
    });

    arena
  }

  #[inline(always)]
  pub fn node_at(&self, idx: usize) -> *const DagNodeCore {
    &self.data[idx]
  }

  #[inline(always)]
  pub fn node_at_mut(&mut self, idx: usize) -> *mut DagNodeCore {
    &mut self.data[idx]
  }

  #[inline(always)]
  pub fn first_node_mut(&mut self) -> *mut DagNodeCore {
    self.node_at_mut(0)
  }
}
