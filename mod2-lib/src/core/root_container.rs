/*!

A `RootContainer` is a linked list of roots of garbage collected objects.

*/

use std::{
  ptr::NonNull,
  sync::{
    atomic::{
      AtomicPtr,
      Ordering
    },
    Mutex
  },
  sync::MutexGuard
};
use crate::api::dag_node::{DagNode, DagNodePtr};

static LIST_HEAD: Mutex<AtomicPtr<RootContainer>> = Mutex::new(AtomicPtr::new(std::ptr::null_mut()));

pub fn acquire_root_list() -> MutexGuard<'static, AtomicPtr<RootContainer>> {
  match LIST_HEAD.try_lock() {
    Ok(lock) => { lock }
    Err(_) => {
      panic!("Deadlocked acquiring root list.")
    }
  }
}

pub struct RootContainer {
  next: Option<NonNull<RootContainer>>,
  prev: Option<NonNull<RootContainer>>,
  node: NonNull<dyn DagNode>
}

unsafe impl Send for RootContainer {}

impl RootContainer {
  pub fn new(node: DagNodePtr) -> Box<RootContainer> {
    assert!(!node.is_null());

    let node: NonNull<dyn DagNode> = NonNull::new(node).unwrap();
    let mut container = Box::new(RootContainer {
      next: None,
      prev: None,
      node
    });
    container.link();
    container
  }

  pub fn mark(&mut self) {
    unsafe {
      self.node.as_mut().mark();
    }
  }

  pub fn link(&mut self){
    let list_head  = acquire_root_list();
    self.prev = None;
    self.next = NonNull::new(list_head.load(Ordering::Relaxed));

    if let Some(mut next) = self.next {
      unsafe {
        next.as_mut().prev = NonNull::new(self);
      }
    }

    list_head.store(self, Ordering::Relaxed);
  }

  pub fn unlink(&mut self){
    let list_head = acquire_root_list();
    if let Some(mut next) = self.next {
      unsafe {
        next.as_mut().prev = self.prev;
      }
    }

    if let Some(mut prev) = self.prev {
      unsafe {
        prev.as_mut().next = self.next;
      }
    } else if let Some(next) = self.next {
      list_head.store(next.as_ptr(), Ordering::Relaxed);
    } else {
      list_head.store(std::ptr::null_mut(), Ordering::Relaxed);
    }
  }

}

impl Drop for RootContainer {
  fn drop(&mut self) {
    self.unlink();
  }
}

/// Marks all roots in the linked list of `RootContainer`s.
pub fn mark_roots() {
  let list_head = acquire_root_list();
  let mut root = unsafe {
    list_head.load(Ordering::Relaxed)
             .as_mut()
             .map(|head| NonNull::new(head as *mut RootContainer).unwrap())
  };

  while let Some(mut root_ptr) = root {
    let root_ref = unsafe{ root_ptr.as_mut() };
    root_ref.mark();
    root = root_ref.next;
  }
}
