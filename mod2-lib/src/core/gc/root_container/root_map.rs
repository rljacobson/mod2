use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Mutex, MutexGuard};
use mod2_abs::{smallvec, SmallVec};
use crate::api::dag_node::{DagNode, DagNodePtr};
use crate::core::gc::node_allocator::NodeAllocator;
use crate::core::gc::root_container::RootSet;
use crate::HashType;

/// Global linked list of `RootMap` nodes
static LIST_HEAD: Mutex<AtomicPtr<RootMap>> = Mutex::new(AtomicPtr::new(std::ptr::null_mut()));

/// Private utility to acquire the root list
fn acquire_root_list() -> MutexGuard<'static, AtomicPtr<RootMap>> {
  match LIST_HEAD.try_lock() {
    Ok(lock) => { lock }
    Err(_) => {
      panic!("Deadlocked acquiring root list.")
    }
  }
}

/// Mark all roots contained in all root maps
pub(super) fn mark_roots() {
  let list_head = acquire_root_list();
  let mut root = unsafe {
    list_head.load(Ordering::Relaxed)
             .as_mut()
             .map(|head| NonNull::new(head as *mut RootMap).unwrap())
  };

  while let Some(mut root_ptr) = root {
    let root_ref = unsafe { root_ptr.as_mut() };
    root_ref.mark();
    root = root_ref.next;
  }
}

pub struct RootMap {
  /// The next root container in the linked list
  pub(super) next: Option<NonNull<RootMap>>,
  /// The previous root container in the linked list
  pub(super) prev: Option<NonNull<RootMap>>,
  /// The vector of root nodes stored in this container
  node_map: HashMap<HashType, DagNodePtr>,

  /// Opt out of `Unpin`
  _pin: std::marker::PhantomPinned,
}

unsafe impl Send for RootMap {}

impl RootMap {
  pub fn new() -> Box<RootMap> {
    let mut container = Box::new(RootMap {
      next : None,
      prev : None,
      node_map: HashMap::default(),
      _pin : std::marker::PhantomPinned::default(),
    });
    container.link();
    container
  }

  /// Mark the nodes contained in this root container as live (during the GC mark phase)
  pub fn mark(&mut self) {
    for (_, node) in self.node_map.iter_mut() {
      node.mark();
    }
  }

  fn link(&mut self){
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

  fn unlink(&mut self){
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

impl Drop for RootMap {
  fn drop(&mut self) {
    self.unlink();
  }
}

impl Deref for RootMap {
  type Target = HashMap<HashType, DagNodePtr>;

  fn deref(&self) -> &Self::Target {
    &self.node_map
  }
}

impl DerefMut for RootMap {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.node_map
  }
}
