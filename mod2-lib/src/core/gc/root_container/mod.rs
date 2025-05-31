/*!

A `RootVec` is a link in the linked list of roots of garbage collected DAG node objects. A `RootSet` is similar
but stores its roots in a `HashSet` rather than a `SmallVec`. A `RootMap` likewise stores its roots in a `HashMap`.

# `RootVec`

The implementation is very efficient in the typical case of storing a single `DagNodePtr`, which it stores inline.
However, if many `DagNodePtr` root objects have the same lifetime, they can be stored in the same `RootVec`, and
the implementation will fall back to a growable vector for storage.

A `RootVec` dereferences to `SmallVec<DagNodePtr, _>`, so it can be treated like a vector.

# `RootSet`

A `RootSet` dereferences to `HashSet<DagNodePtr>`, so it can be treated like a `HashSet`.

# `RootMap`

A `RootMap` dereferences to `HashMap<HashValueType, DagNodePtr>`, so it can be treated like a `HashMap`.

*/

mod root_map;
mod root_set;
mod root_vec;

use std::{
  ops::{Deref, DerefMut},
  ptr::NonNull,
  sync::atomic::Ordering,
};
use crate::api::dag_node::DagNode;

// These always need to be boxed
pub use root_map::RootMap;
pub use root_set::RootSet;
pub use root_vec::RootVec;
pub type BxRootMap = Box<RootMap>;
pub type BxRootSet = Box<RootSet>;
pub type BxRootVec = Box<RootVec>;

/// Marks all roots in the linked lists of root containers.
pub fn mark_roots() {
  root_map::mark_roots();
  root_set::mark_roots();
  root_vec::mark_roots();
}
