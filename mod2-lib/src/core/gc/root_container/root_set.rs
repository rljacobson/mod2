use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Mutex, MutexGuard};
use mod2_abs::{smallvec, SmallVec};
use crate::api::dag_node::{DagNode, DagNodePtr};
use crate::core::gc::root_container::root_map::RootMap;

/// Global linked list of `RootSet` nodes
static LIST_HEAD: Mutex<AtomicPtr<RootSet>> = Mutex::new(AtomicPtr::new(std::ptr::null_mut()));

/// Private utility to acquire the root list
pub fn acquire_root_list() -> MutexGuard<'static, AtomicPtr<RootSet>> {
  match LIST_HEAD.try_lock() {
    Ok(lock) => { lock }
    Err(_) => {
      panic!("Deadlocked acquiring root list.")
    }
  }
}

/// Mark all roots contained in all root sets
pub(super) fn mark_roots() {
  let list_head = acquire_root_list();
  let mut root = unsafe {
    list_head.load(Ordering::Relaxed)
             .as_mut()
             .map(|head| NonNull::new(head as *mut RootSet).unwrap())
  };

  while let Some(mut root_ptr) = root {
    let root_ref = unsafe { root_ptr.as_mut() };
    root_ref.mark();
    root = root_ref.next;
  }
}

pub struct RootSet {
  /// The next root container in the linked list
  pub(super) next: Option<NonNull<RootSet>>,
  /// The previous root container in the linked list
  pub(super) prev: Option<NonNull<RootSet>>,
  /// The vector of root nodes stored in this container
  nodes: HashSet<DagNodePtr>,

  /// Opt out of `Unpin`
  _pin: std::marker::PhantomPinned,
}

unsafe impl Send for RootSet {}

impl RootSet {
  pub fn new() -> Box<RootSet> {
    let mut container = Box::new(RootSet {
      next : None,
      prev : None,
      nodes: HashSet::default(),
      _pin : std::marker::PhantomPinned::default(),
    });
    container.link();
    container
  }

  /// Mark the nodes contained in this root container as live (during the GC mark phase)
  pub fn mark(&mut self) {
    for mut node in self.nodes.iter().cloned() {
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

impl Drop for RootSet {
  fn drop(&mut self) {
    self.unlink();
  }
}

impl Deref for RootSet {
  type Target = HashSet<DagNodePtr>;

  fn deref(&self) -> &Self::Target {
    &self.nodes
  }
}

impl DerefMut for RootSet {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.nodes
  }
}
