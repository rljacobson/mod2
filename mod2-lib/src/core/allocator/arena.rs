/*!

An arena allocator for `DagNode`s.

*/

use std::{
  mem::MaybeUninit,
  ptr::null_mut
};

use crate::{
  core::dag_node_core::DagNodeCore,
  core::{
    allocator::node_allocator::ARENA_SIZE,
  }
};

#[repr(align(8))]
pub struct Arena {
  pub(crate) next_arena: *mut Arena,
  data: [DagNodeCore; ARENA_SIZE],
}

impl Arena {
  #[inline(always)]
  pub fn allocate_new_arena() -> *mut Arena {

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
      next_arena: null_mut(),
      data      : unsafe { std::mem::transmute::<_, [DagNodeCore; ARENA_SIZE]>(data) }
    });

    Box::into_raw(arena)
  }

  #[inline(always)]
  pub fn first_node(&mut self) -> *mut DagNodeCore {
    &mut self.data[0]
  }
}
