/*!

A `Bucket` is a small arena. We might use bumpalo or something instead.

Note that as long as the data itself isn't moved, bucket can be moved.

*/

use std::ptr::{null_mut, NonNull};
use crate::core::Byte;

pub struct Bucket {
  pub(crate) data         : Box<[Byte]>,
  pub(crate) next_free_idx: usize,
}

impl Bucket {
  pub fn with_capacity(capacity: usize) -> Self {
    Bucket {
      data         : vec![0; capacity].into_boxed_slice(),
      next_free_idx: 0,
    }
  }
  
  pub fn bytes_free(&self) -> usize {
    if self.next_free_idx <= self.data.len(){
      self.data.len() - self.next_free_idx
    } else {
      0
    }
  }

  pub fn allocate(&mut self, bytes_needed: usize) -> *mut Byte {
    assert!(self.bytes_free() >= bytes_needed);
    let allocation = self.next_free_idx;
    
    // `self.next_free_idx` is always aligned on an 8 byte boundary. If the offset would escape the allocation, 
    // `align_offset` returns `usize::MAX`, `bytes_used` is saturated, and thus `self.next_free_idx`.
    let align_offset = (&self.data[self.next_free_idx + bytes_needed] as *const Byte).align_offset(8);
    let bytes_used   = bytes_needed + align_offset;
    self.next_free_idx += bytes_used;

    &mut self.data[allocation]
  }

  pub fn reset(&mut self) {
    self.next_free_idx = 0;
  }
}
