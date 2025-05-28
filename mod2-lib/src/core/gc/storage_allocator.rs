/*!

# Bucket Allocator

See GarbageCollector.md for a detailed explanation of how it works. Below is a brief summary of how it works.

The Bucket allocator manages memory by organizing it into buckets, each containing raw memory that
can be allocated in smaller chunks. When a program requests memory, the allocator first searches the
in-use buckets for a free chunk. In the typical case, the current active bucket has the capacity to
allocate the requested chunk, and so the allocator acts as a "bump" allocator. If no suitable space
is found, it checks unused buckets (if any exist) or allocates a new one to accommodate the request.

The garbage collection process in the bucket allocator follows a mark-and-sweep pattern
with a copying strategy. During the mark phase, the allocator traverses the live data
and copies it to available initially empty buckets (i.e. buckets which were empty prior
to garbage collection). If the available buckets do not have enough space to accommodate
the live objects, new buckets are allocated and added to the list. Once the objects
are copied, the old memory locations are free to be collected in the sweep phase.

In the sweep phase, the allocator clears the old buckets, resetting their free
space to the full bucket size. These buckets are then moved to the unused list
and reset to an empty state, making them available for future allocations.

Because live objects are relocated during garbage collection to previously empty buckets,
there is no fragmentation after garbage collection. What's more, copying occurs in
depth-first order on the graph nodes, improving locality for certain access patterns.

*/

use std::{
  cmp::max,
  sync::{Mutex, MutexGuard},
  ptr::NonNull
};

use once_cell::sync::Lazy;
use mod2_abs::{debug, heap_construct, info};
use crate::{
  core::{
    gc::bucket::Bucket,
    Byte
  }
};


/// To determine bucket size for huge allocations
const BUCKET_MULTIPLIER: usize = 8;
/// Bucket size for normal allocations
const MIN_BUCKET_SIZE  : usize = 256 * 1024 - 8;
/// Just under 8/9 of MIN_BUCKET_SIZE
const INITIAL_TARGET   : usize = 220 * 1024;
const TARGET_MULTIPLIER: usize = 8;

static GLOBAL_STORAGE_ALLOCATOR: Lazy<Mutex<StorageAllocator>> = Lazy::new(|| {
  Mutex::new(StorageAllocator::new())
});

pub fn acquire_storage_allocator()  -> MutexGuard<'static, StorageAllocator> {
  GLOBAL_STORAGE_ALLOCATOR.lock().unwrap()
}

pub struct StorageAllocator {
  /// Buckets in use
  buckets_in_use: Vec<Bucket>,
  /// Unused buckets
  unused_buckets: Vec<Bucket>,

  /// Buckets in use during mark phase
  buckets_being_marked: Vec<Bucket>,

  // General settings
  show_gc: bool, // Do we report GC stats to user
  need_to_collect_garbage: bool,

  // Bucket management variables
  /// Amount of bucket storage in use (bytes)
  storage_in_use       : usize,
  /// Total amount of bucket storage (bytes)
  total_bytes_allocated: usize,
  /// A temporary to remember storage use prior to GC
  old_storage_in_use   : usize,
  /// Amount of storage in use before GC (bytes)
  target               : usize,
}

// Access is hidden behind a mutex.
unsafe impl Send for StorageAllocator {}
// unsafe impl Sync for Allocator {}

impl StorageAllocator {
  pub fn new() -> Self {
    StorageAllocator {
      buckets_in_use: vec![],
      unused_buckets: vec![],
      buckets_being_marked: vec![],

      show_gc: true,
      need_to_collect_garbage: false,

      storage_in_use       : 0,
      total_bytes_allocated: 0,
      old_storage_in_use   : 0,
      target               : INITIAL_TARGET,
    }
  }

  /// Query whether the allocator has any garbage to collect.
  #[inline(always)]
  pub fn want_to_collect_garbage(&self) -> bool {
    self.need_to_collect_garbage
  }

  /// Allocates the given number of bytes using bucket storage.
  pub fn allocate_storage(&mut self, bytes_needed: usize) -> *mut Byte {
    assert_eq!(bytes_needed % size_of::<usize>(), 0, "only whole machine words can be allocated");
    self.storage_in_use += bytes_needed;

    if self.storage_in_use > self.target {
      self.need_to_collect_garbage = true;
    }

    for bucket in self.buckets_in_use.iter_mut() {
      if bucket.bytes_free() >= bytes_needed {
        return bucket.allocate(bytes_needed);
      }
    }

    // No space in any bucket, so we need to allocate a new one.
    self.slow_allocate_storage(bytes_needed)
  }

  /// Allocates the given number of bytes by creating more bucket storage.
  fn slow_allocate_storage(&mut self, bytes_needed: usize) -> *mut u8 {
    #[cfg(feature = "gc_debug")]
    {
      debug!(2, "slow_allocate_storage()");
    }

    // Find an unused bucket with enough storage for the allocation
    if let Some(bucket_idx) = self.unused_buckets.iter().position(|bucket| bucket.bytes_free() >= bytes_needed) {
      // Remove from unused list
      let mut bucket = self.unused_buckets.swap_remove(bucket_idx);

      // Allocate storage from bucket
      let allocation = bucket.allocate(bytes_needed);
      // Place bucket in used list
      self.buckets_in_use.push(bucket);

      return allocation;
    }

    // No suitable unused bucket found. Create a new bucket.

    let new_bucket_size = usize::max(BUCKET_MULTIPLIER * bytes_needed, MIN_BUCKET_SIZE);
    let mut new_bucket  = Bucket::with_capacity(new_bucket_size);
    let allocation      = new_bucket.allocate(bytes_needed);

    self.total_bytes_allocated += new_bucket_size;
    self.buckets_in_use.push(new_bucket);

    allocation
  }

  /// Prepare bucket storage for mark phase of GC
  pub(crate) fn _prepare_to_mark(&mut self) {
    self.old_storage_in_use = self.storage_in_use;
    // Buckets being marked is empty before this swap.
    std::mem::swap(&mut self.buckets_in_use, &mut self.buckets_being_marked);

    // New allocations occur from whatever buckets are in unused list. We rely on the usual
    // `slow_allocate_storage()` algorithm to draw buckets from the unused list as needed.

    self.storage_in_use          = 0;
    self.need_to_collect_garbage = false;
  }

  /// Garbage Collection for Buckets, called after mark completes
  pub(crate) fn _sweep_garbage(&mut self) {
    // Now we sweep all buckets in `self.buckets_being_marked`.

    // Reset all formerly active buckets and move them to the list of unused buckets.
    self.unused_buckets.extend(
      self.buckets_being_marked
          .drain(..)
          .map(|mut bucket| { bucket.reset(); bucket })
    );
    // Now `self.buckets_being_marked` is back to being empty, ready for the next collection.
    self.target = max(self.target, TARGET_MULTIPLIER*self.storage_in_use);

    if self.show_gc {
      let bucket_count = self.buckets_in_use.len() + self.unused_buckets.len();

      info!(1,
        "{:<10} {:<10} {:<10} {:<10} {:<13} {:<10} {:<10} {:<10} {:<10}",
        "Buckets",
        "Bytes",
        "Size (MB)",
        "In use",
        "In use (MB)",
        "Collected",
        "Col. (MB)",
        "Now",
        "Now (MB)"
      );
      info!(1,
        "{:<10} {:<10} {:<10.2} {:<10} {:<13.2} {:<10} {:<10.2} {:<10.2} {:<10.2}",
        bucket_count,
        self.total_bytes_allocated,
        (self.total_bytes_allocated as f64) / (1024.0 * 1024.0),
        self.old_storage_in_use,
        (self.old_storage_in_use as f64) / (1024.0 * 1024.0),
        self.old_storage_in_use - self.storage_in_use,
        ((self.old_storage_in_use - self.storage_in_use) as f64) / (1024.0 * 1024.0),
        self.storage_in_use,
        (self.storage_in_use as f64) / (1024.0 * 1024.0),
      );
    }

  }

}

