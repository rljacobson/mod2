/*!

# Arena Allocator
See GarbageCollector.md for a detailed explanation of how it works. Below is a brief summary of how it works.

The arena allocator manages memory by organizing it into arenas, which are fixed size arrays of nodes available for allocation. The allocator uses a simple mark-and-sweep algorithm to collect garbage, but the sweep phase is "lazy." When the program requests a new node allocation, the allocator searches linearly for free nodes within these arenas and reuses them when possible. During this linear search, the allocator performs a "lazy sweep," clearing all "marked" flags on nodes and running destructors when necessary. This proceeds until either an available node is found and returned or all nodes are found to be in use, in which case it may expand by creating a new arena or adding capacity to existing ones.

When garbage collection is triggered, the allocator then sweeps the remaining (not yet searched) part of the arena(s). Then it begins the mark phase. During marking, the allocator requests all node roots to flag nodes that are actively in use so that they’re preserved. During this phase, the number of active nodes is computed. After marking, the allocator compares it's total node capacity to the number of active nodes and, if the available capacity is less than a certain "slop factor," more arenas are allocated from system memory. The "cursor" for the linear search is then reset to the first node of the first arena.

Since the sweep phase is done lazily, the time it takes to sweep the arenas is amortized between garbage collection events. Because garbage collection is triggered when the linear search for free nodes nears the end of the last arena, allocating a "slop factor" of extra arenas keeps garbage collection events low.

*/

use std::{
  sync::{
    atomic::{
      Ordering::Relaxed,
      AtomicUsize
    },
    Mutex,
    MutexGuard,
  },
  ptr::{
    drop_in_place,
    null_mut
  }
};
use once_cell::sync::Lazy;
use mod2_abs::{debug, info};

use crate::{
  core::{
    dag_node_core::{
      ThinDagNodePtr,
      DagNodeCore,
      DagNodeFlag,
      DagNodeFlags,
    },
    gc::{
      arena::Arena,
      storage_allocator::acquire_storage_allocator,
      root_container::mark_roots,
    }
  }
};

// Constant Allocator Parameters
const SMALL_MODEL_SLOP: f64   = 8.0;
const BIG_MODEL_SLOP  : f64   = 2.0;
const LOWER_BOUND     : usize =  4 * 1024 * 1024; // Use small model if <= 4 million nodes
const UPPER_BOUND     : usize = 32 * 1024 * 1024; // Use big model if >= 32 million nodes
// It looks like Maude assumes DagNodes are 6 words in size, but ours are 3 words,
// at least so far.
pub(crate) const ARENA_SIZE: usize = 5460; // Arena size in nodes; 5460 * 6 + 1 + new/malloc_overhead <= 32768 words
const RESERVE_SIZE         : usize = 256; // If fewer nodes left call GC when allowed


pub(crate) static ACTIVE_NODE_COUNT: AtomicUsize = AtomicUsize::new(0);
static GLOBAL_NODE_ALLOCATOR: Lazy<Mutex<NodeAllocator>> = Lazy::new(|| {
  Mutex::new(NodeAllocator::new())
});

/// Acquire the global node allocator. The `caller_msg` is for debugging purposes.
#[inline(always)]
pub fn acquire_node_allocator(caller_msg: &str) -> MutexGuard<'static, NodeAllocator> {
  GLOBAL_NODE_ALLOCATOR.lock().expect(caller_msg)
}

#[inline(always)]
pub fn ok_to_collect_garbage() {
  acquire_node_allocator("ok_to_collect_garbage").ok_to_collect_garbage();
}

#[inline(always)]
pub fn want_to_collect_garbage() -> bool {
  acquire_node_allocator("want_to_collect_garbage").want_to_collect_garbage()
}

#[inline(always)]
pub fn allocate_dag_node() -> ThinDagNodePtr {
  acquire_node_allocator("want_to_collect_garbage").allocate_dag_node()
}


pub(crate) struct NodeAllocator {
  arenas : Vec<Box<Arena>>,
  
  // General settings
  show_gc: bool, // Do we report GC stats to user
  need_to_collect_garbage: bool,

  // Arena management variables
  current_arena_past_active_arena: bool,
  current_arena_idx              : usize,
  next_node_idx                  : usize,
  end_idx                        : usize,
  last_active_arena_idx          : usize,
  last_active_node_idx           : usize,
}

// Access is hidden behind a mutex.
unsafe impl Send for NodeAllocator {}
// unsafe impl Sync for Allocator {}

impl NodeAllocator {
  pub fn new() -> Self {
    NodeAllocator {
      arenas: vec![],
      
      show_gc: true,
      need_to_collect_garbage: false,

      current_arena_past_active_arena: true,
      current_arena_idx              : 0,
      next_node_idx                  : 0,
      end_idx                        : 0,
      last_active_arena_idx          : 0,
      last_active_node_idx           : 0,
    }
  }

  /// Tell the garbage collector to collect garbage if it needs to.
  /// You can query whether it needs to by calling `want_to_collect_garbage`,
  /// but this isn't necessary.
  #[inline(always)]
  pub fn ok_to_collect_garbage(&mut self) {
    if self.need_to_collect_garbage {
      unsafe{ self.collect_garbage(); }
    }
  }

  /// Query whether the allocator has any garbage to collect.
  #[inline(always)]
  pub fn want_to_collect_garbage(&self) -> bool {
    self.need_to_collect_garbage
  }
  
  /// Get the given node at the current arena
  #[inline(always)]
  pub fn node_at(&mut self, idx: usize) -> ThinDagNodePtr {
    let arena = self.arenas[self.current_arena_idx].as_mut();
    arena.node_at_mut(idx)
  }

  /// Allocates a new `DagNode`
  pub fn allocate_dag_node(&mut self) -> ThinDagNodePtr {
    let mut current_node_idx = self.next_node_idx;

    unsafe{
      loop {
        if current_node_idx == self.end_idx {
          // Arena is full or no arena has been allocated. Allocate a new one.
          current_node_idx = self.slow_new_dag_node();
          break;
        }

        { // Scope of `current_node_mut: &mut DagNode`
          let current_node_mut = self.node_at(current_node_idx).as_mut_unchecked();
          if !current_node_mut.is_marked() {
            if current_node_mut.needs_destruction() {
              drop_in_place(current_node_mut);
            }
            break;
          }
          current_node_mut.flags.remove(DagNodeFlag::Marked);
          // current_node_mut.flags = DagNodeFlags::default();
        }

        current_node_idx += 1;
      } // end loop

      // initialize/reset
      let current_node_ptr = self.node_at(current_node_idx);
      current_node_ptr.as_mut_unchecked().args = null_mut();
      self.next_node_idx = current_node_idx + 1;
      
    } // end of unsafe block

    increment_active_node_count();
    self.node_at(current_node_idx)
  }


  /// Allocates a new arena, adding it to the linked list of arenas, and
  /// returns (a pointer to) the new arena.
  fn allocate_new_arena(&mut self) {
    #[cfg(feature = "gc_debug")]
    {
      debug!(2, "allocate_new_arena()");
      self.dump_memory_variables();
    }
    self.arenas.push(Arena::allocate_new_arena());
  }

  /// Allocate a new `DagNode` when the current arena is (almost) full. Returns the index into the current arena.
  unsafe fn slow_new_dag_node(&mut self) -> usize {
    #[cfg(feature = "gc_debug")]
    {
      debug!(2, "slow_new_dag_node()");
      self.dump_memory_variables();
    }

    loop {
      if self.arenas.is_empty() {
        // Allocate the first arena
        self.allocate_new_arena();
        // The last arena in the linked list is given a reserve.
        self.end_idx   = ARENA_SIZE - RESERVE_SIZE;

        // These two members are initialized on first call to `NodeAllocator::sweep_arenas()`.
        // self.last_active_arena = arena;
        // self.last_active_node  = first_node;

        // The index of the first node in the new arena. Note that `self.current_arena_idx` is already `0`.
        return 0;
      }

      // If the current arena is the last one
      if self.current_arena_idx == self.arenas.len() - 1 {
        self.need_to_collect_garbage = true;

        if self.end_idx != ARENA_SIZE {
          // Use up the reserve
          self.next_node_idx = self.end_idx; // Next node is invalid where we are called.
          self.end_idx       = ARENA_SIZE;
        } else {
          // Allocate a new arena
          if self.current_arena_idx == self.last_active_arena_idx {
            self.current_arena_past_active_arena = true;
          }

          self.current_arena_idx = self.arenas.len(); 
          self.allocate_new_arena();
          self.end_idx   = ARENA_SIZE; // ToDo: Why no reserve here?

          // The index of the first node in the new arena
          return 0;
        }
      } // end if current arena is last
      else {
        // Use next arena
        if self.current_arena_idx == self.last_active_arena_idx {
          self.current_arena_past_active_arena = true;
        }

        self.current_arena_idx += 1;
        self.next_node_idx = 0;

        match self.current_arena_idx == self.arenas.len() - 1 {
          true => {
            // The last arena in the linked list is given a reserve.
            self.end_idx = ARENA_SIZE - RESERVE_SIZE;
          }
          false => {
            self.end_idx = ARENA_SIZE;
          }
        }
      }

      #[cfg(feature = "gc_debug")]
      self.check_invariant();

      // Now execute lazy sweep to actually find a free location. Note that this is the same code as in
      // `allocate_dag_node`, except there is no `slow_new_dag_node` case.

      let arena = self.arenas[self.current_arena_idx].as_mut();

      // Loop over all nodes from self.next_node_idx to self.end_idx
      for cursor_idx in self.next_node_idx..self.end_idx {
        let cursor_ptr = arena.node_at_mut(cursor_idx);
        let cursor_mut = cursor_ptr.as_mut_unchecked();

        if !cursor_mut.is_marked(){
          if cursor_mut.needs_destruction(){
            drop_in_place(cursor_ptr);
          }
          return cursor_idx;
        }

        cursor_mut.flags.remove(DagNodeFlag::Marked);

      } // end loop over all nodes
    } // end outermost loop
  }

  // The pub(super) is only for testing.
  pub(super) unsafe fn collect_garbage(&mut self) {
    static mut GC_COUNT: u64 = 0;

    if self.arenas.is_empty() {
      return;
    }

    GC_COUNT += 1;
    let gc_count = GC_COUNT; // To silence shared_mut_ref warning
    if self.show_gc {
      // We moved this up here so that it appears before the bucket storage statistics.
      println!("Collection: {}", gc_count);
    }

    self.sweep_arenas();
    #[cfg(feature = "gc_debug")]
    self.check_arenas();

    // Mark phase

    let old_active_node_count = active_node_count();
    ACTIVE_NODE_COUNT.store(0, Relaxed); // to be updated during mark phase.

    acquire_storage_allocator()._prepare_to_mark();

    mark_roots();

    acquire_storage_allocator()._sweep_garbage();

    // Garbage Collection for Arenas
    let active_node_count = active_node_count(); // updated during mark phase

    let node_capacity = self.arenas.len() * ARENA_SIZE;

    if self.show_gc {
      info!(1,
        "{:<10} {:<10} {:<10} {:<10} {:<13} {:<10} {:<10} {:<10} {:<10}",
        "Arenas",
        "Nodes",
        "Size (MB)",
        "In use",
        "In use (MB)",
        "Collected",
        "Col. (MB)",
        "Now",
        "Now (MB)"
      );
      info!(1,
        "{:<10} {:<10} {:<10.2} {:<10} {:<13.2} {:<10} {:<10.2} {:<10} {:<10.2}",
        self.arenas.len(),
        node_capacity,
        ((node_capacity * size_of::<DagNodeCore>()) as f64) / (1024.0 * 1024.0),
        old_active_node_count,
        (((old_active_node_count) * size_of::<DagNodeCore>()) as f64) / (1024.0 * 1024.0),
        old_active_node_count - active_node_count,
        (((old_active_node_count - active_node_count) * size_of::<DagNodeCore>()) as f64) / (1024.0 * 1024.0),
        active_node_count,
        ((active_node_count * size_of::<DagNodeCore>()) as f64) / (1024.0 * 1024.0),
      );
    }

    // Calculate if we should allocate more arenas to avoid an early gc.
    // Compute slop factor
    // Case: ACTIVE_NODE_COUNT >= UPPER_BOUND
    let mut slop_factor: f64 = BIG_MODEL_SLOP;
    if ACTIVE_NODE_COUNT.load(Relaxed) < LOWER_BOUND {
      // Case: ACTIVE_NODE_COUNT < LOWER_BOUND
      slop_factor = SMALL_MODEL_SLOP;
    } else if ACTIVE_NODE_COUNT.load(Relaxed) < UPPER_BOUND {
      // Case: LOWER_BOUND <= ACTIVE_NODE_COUNT < UPPER_BOUND
      // Linearly interpolate between the two models.
      slop_factor += ((UPPER_BOUND - active_node_count as usize) as f64 * (SMALL_MODEL_SLOP - BIG_MODEL_SLOP)) / (UPPER_BOUND - LOWER_BOUND) as f64;
    }

    // Allocate new arenas so that we have capacity for at least slop_factor times the actually used nodes.
    let ideal_arena_count = (active_node_count as f64 * slop_factor / (ARENA_SIZE as f64)).ceil() as usize;

    #[cfg(feature = "gc_debug")]
    debug!(2, "ideal_arena_count: {}", ideal_arena_count);
    while self.arenas.len() < ideal_arena_count {
      self.allocate_new_arena();
    }

    // Reset state variables
    self.current_arena_past_active_arena = false;
    self.current_arena_idx = 0;
    self.next_node_idx = 0;
    match self.current_arena_idx == self.arenas.len() - 1 {
      true => {
        // The last arena in the linked list is given a reserve.
        self.end_idx = ARENA_SIZE - RESERVE_SIZE;
      },
      false => {
        self.end_idx = ARENA_SIZE;
      }
    }
    self.need_to_collect_garbage = false;

    #[cfg(feature = "gc_debug")]
    {
      debug!(2, "end of GC");
      self.dump_memory_variables();
    }
  }

  /// Tidy up lazy sweep phase - clear marked flags and call dtors where necessary.
  unsafe fn sweep_arenas(&mut self) {
    #[cfg(feature = "gc_debug")]
    {
      debug!(2, "sweep_arenas()");
      self.dump_memory_variables();
    }

    let mut new_last_active_arena = self.current_arena_idx;
    // self.next_node never points to first node, so subtract 1.
    let mut new_last_active_node  = self.next_node_idx - 1;

    // `NodeAllocator::current_arena_past_active_arena` is initialized to `true`, so this whole method
    // effectively just initializes `last_active_arena_idx` and `last_active_node_idx`.
    if !self.current_arena_past_active_arena {
      // First tidy arenas from current up to last_active.
      let mut node_cursor  = self.next_node_idx;
      let mut arena_cursor = self.current_arena_idx;

      while arena_cursor != self.last_active_arena_idx {
        let end_node_idx = ARENA_SIZE;

        while node_cursor != end_node_idx {
          let node_cursor_ptr = self.arenas[arena_cursor].node_at_mut(node_cursor);
          let node_cursor_mut = node_cursor_ptr.as_mut_unchecked();

          if node_cursor_mut.is_marked() {
            new_last_active_arena = arena_cursor;
            new_last_active_node  = node_cursor;
            node_cursor_mut.flags.remove(DagNodeFlag::Marked);
          }
          else {
            if node_cursor_mut.needs_destruction() {
              drop_in_place(node_cursor_ptr);
            }
            node_cursor_mut.flags = DagNodeFlags::default();
          }

          node_cursor += 1;
        } // end loop over nodes

        arena_cursor += 1;
        node_cursor = 0;
      } // end loop over arenas

      // Now tidy last_active_arena from d up to and including last_active_node_idx.
      let end_node_idx = self.last_active_node_idx;

      while node_cursor <= end_node_idx {
        let node_cursor_ptr = self.arenas[arena_cursor].node_at_mut(node_cursor);
        let node_cursor_mut = node_cursor_ptr.as_mut_unchecked();

        if node_cursor_mut.is_marked() {
          new_last_active_arena = arena_cursor;
          new_last_active_node  = node_cursor;
          node_cursor_mut.flags.remove(DagNodeFlag::Marked);
        }
        else {
          if node_cursor_mut.needs_destruction() {
            drop_in_place(node_cursor_ptr);
          }
          node_cursor_mut.flags = DagNodeFlags::default();
        }

        node_cursor += 1;
      } // end loop over active nodes
    }

    self.last_active_arena_idx = new_last_active_arena;
    self.last_active_node_idx  = new_last_active_node;
  }

  /// Verify that no `DagNode` objects within the arenas managed by the allocator are in a “marked” state.
  #[cfg(feature = "gc_debug")]
  unsafe fn check_invariant(&self) {
    for (arena_idx, arena) in self.arenas.iter().enumerate() {
      let bound: usize =
          match arena_idx == self.current_arena_idx {

            true => self.next_node_idx,

            false => ARENA_SIZE

          };

      for node_idx in 0..bound {
        if arena.node_at(node_idx).as_ref_unchecked().is_marked() {
          debug!(2, "check_invariant() : MARKED DagNode! arena = {} node = {}", arena_idx, node_idx);
        }
      } // end loop over nodes

      if arena_idx == self.current_arena_idx { break; }
    } // end loop over arenas
  }

  #[cfg(feature = "gc_debug")]
  unsafe fn check_arenas(&self) {
    for (arena_idx, arena) in self.arenas.iter().enumerate() {
      for node_idx in 0..ARENA_SIZE {
        if arena.node_at(node_idx).as_ref_unchecked().is_marked()  {
          debug!(2, "check_arenas() : MARKED DagNode! arena = {} node = {}", arena_idx, node_idx);
        }
      } // end loop over nodes

      if arena_idx == self.current_arena_idx { break; }
    } // end loop over arenas
  }

  /// Prints the state of the allocator.
  #[cfg(feature = "gc_debug")]
  pub fn dump_memory_variables(&self) {
    let bucket_needs_collection = acquire_storage_allocator().want_to_collect_garbage();

    //────────
    eprintln!("╭─────────────────────────────────────────────╮");
    eprintln!("│{:<32} {:>12}│", "Variable", "Value");
    eprintln!("├─────────────────────────────────────────────┤");
    eprintln!("│{:<32} {:>12}│", "arena_count", self.arenas.len());
    eprintln!("│{:<32} {:>12}│", "active_node_count", ACTIVE_NODE_COUNT.load(Relaxed));
    eprintln!("│{:<32} {:>12}│", "need_to_collect_garbage", self.need_to_collect_garbage);
    eprintln!(
      "│{:<32} {:>12}│",
      "need_to_collect_storage",
      bucket_needs_collection
    );
    eprintln!(
      "│{:<32} {:>12}│",
      "current_arena_past_active_arena",
      self.current_arena_past_active_arena
    );
    eprintln!(
      "│{:<32} {:>12}│",
      "current_arena",
      self.current_arena_idx
    );
    eprintln!(
      "│{:<32} {:>12}│",
      "next_node",
      self.next_node_idx
    );
    eprintln!(
      "│{:<32} {:>12}│",
      "end_pointer",
      self.end_idx
    );
    eprintln!(
      "│{:<32} {:>12}│",
      "last_active_arena",
      self.last_active_arena_idx
    );
    eprintln!(
      "│{:<32} {:>12}│",
      "last_active_node",
      self.last_active_node_idx
    );
    eprintln!("╰─────────────────────────────────────────────╯");
  }
}




#[inline(always)]
pub(crate) fn increment_active_node_count() {
  ACTIVE_NODE_COUNT.fetch_add(1, Relaxed);
}

#[inline(always)]
pub fn active_node_count() -> usize {
  ACTIVE_NODE_COUNT.load(Relaxed)
}

