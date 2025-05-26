/*!

A `RootContainer` is a link in the linked list of roots of garbage collected DAG node objects.

The implementation is very efficient in the typical case of storing a single `DagNodePtr`, which it stores inline.
However, if many `DagNodePtr` root objects have the same lifetime, they can be stored in the same `RootContainer`, and
the implementation will fall back to a growable vector for storage.

A `RootContainer` dereferences to `SmallVec<DagNodePtr, _>`, so it can be treated like a vector.

*/

use std::{
  ops::{Deref, DerefMut},
  ptr::NonNull,
  sync::{
    atomic::{
      AtomicPtr,
      Ordering
    },
    Mutex,
    MutexGuard
  },
};
use mod2_abs::{smallvec, SmallVec};
use crate::api::dag_node::{DagNode, DagNodePtr};

/// Private global linked list of root nodes
static LIST_HEAD: Mutex<AtomicPtr<RootContainer>> = Mutex::new(AtomicPtr::new(std::ptr::null_mut()));

/// Private utility to acquire the root list 
fn acquire_root_list() -> MutexGuard<'static, AtomicPtr<RootContainer>> {
  match LIST_HEAD.try_lock() {
    Ok(lock) => { lock }
    Err(_) => {
      panic!("Deadlocked acquiring root list.")
    }
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

// Local convenience type
type SmallNodeVec = SmallVec<DagNodePtr, 1>;


pub struct RootContainer {
  /// The next `RootContainer` in the linked list
  next: Option<NonNull<RootContainer>>,
  /// The previous `RootContainer` in the linked list
  prev: Option<NonNull<RootContainer>>,
  /// The vector of root nodes stored in this container
  nodes: SmallNodeVec,
  
  /// Opt out of `Unpin`
  _pin: std::marker::PhantomPinned,
}

unsafe impl Send for RootContainer {}

impl RootContainer {
  pub fn new() -> Box<RootContainer> {
    let mut container = Box::new(RootContainer {
      next : None,
      prev : None,
      nodes: SmallVec::new(),
      _pin : std::marker::PhantomPinned::default(),
    });
    container.link();
    container
  }
  
  /// Construct a new `RootContainer` containing the given node
  pub fn with_node(node: DagNodePtr) -> Box<RootContainer> {
    let mut container = Box::new(RootContainer {
      next : None,
      prev : None,
      nodes: smallvec![node],
      _pin : std::marker::PhantomPinned::default(),
    });
    container.link();
    container
  }

  /// Mark the nodes contained in this `RootContainer` as live (during the GC mark phase) 
  pub fn mark(&mut self) {
    for node in &mut self.nodes {
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

impl Drop for RootContainer {
  fn drop(&mut self) {
    self.unlink();
  }
}

impl Deref for RootContainer {
  type Target = SmallNodeVec;

  fn deref(&self) -> &Self::Target {
    &self.nodes
  }
}

impl DerefMut for RootContainer {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.nodes
  }
}