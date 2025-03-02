/*!
The allocator for garbage collected memory. This is really two different allocators which collect garbage at the same time:

 1. An arena allocator exclusively for allocating `DagNode` objects. All garbage collected nodes must be allocated with this allocator.
 2. A "bucket" allocator exclusively for allocating any memory owned by `DagNode` objects. Nodes may have several arguments, which are other nodes. The arguments are stored as arrays of pointers to the argument nodes, and nodes must allocate these arrays of pointers using the bucket allocator and hold on to a pointer to the array.


*/
#![allow(unused_imports)]
mod arena;
mod bucket;
pub(crate) mod gc_vector;
mod node_allocator;
mod storage_allocator;

#[cfg(test)]
mod tests;

// Used internally
use node_allocator::acquire_node_allocator;
use storage_allocator::acquire_storage_allocator;
// Needed within all node `mark()` methods
pub(crate) use node_allocator::increment_active_node_count;

// These are the only public API
pub use node_allocator::{
  ok_to_collect_garbage,
  want_to_collect_garbage,
  allocate_dag_node
};


