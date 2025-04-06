#![feature(ptr_metadata)]
#![allow(unused)]
/*!

Types/type aliases that abstract over the implementing backing type.

# Background and Motivation

A motivating example is the `IString` type, an interned string type. A number of external crates could provide this
functionality. This module redirects to whatever chosen implementation we want. To use the
[`string_cache` crate](https://crates.io/crates/string_cache), we just define `IString` as an alias for
`string_cache::DefaultAtom`:

```ignore
pub use string_cache::DefaultAtom as IString;
```

If we want to later change to the [`ustr` crate](https://crates.io/crates/ustr), we just define `IString` to be an
alias for `ustr::Ustr` instead:

```ignore
pub use ustr::Ustr as IString;
```

The `ustr` and `string_cache` crates conveniently have very similar public APIs. For types or infrastructure with very
different backing implementations, we define an abstraction layer over the implementation. For example, the `log`
module could use any of a number of logging frameworks or even a bespoke solution for its implementation. However, its
(crate) public interface consists only of `set_global_logging_threshold()`/`get_global_logging_threshold()` and the
macros `critical!`, `error!`, `warning!`, `info!`, `debug!`, and `trace!`. The (private) backing implementation is
encapsulated in the `log` module.

*/

mod erased;
mod graph;
mod heap;
mod nat_set;
mod memory;
mod rccell;
mod string_util;
mod unsafe_ptr;
mod partial_ordering;
pub mod any;
pub mod hash;

// Generic memory utilities
pub use memory::as_bytes;

// Aliases and utility 
pub use partial_ordering::*;

// Arbitrary precision arithmetic
pub mod numeric;

// region Hashing data structures
use std::collections::HashSet as StdHashSet;
use std::collections::HashMap as StdHashMap;
pub use std::collections::HashSet;
pub use std::collections::HashMap;

// For vectors that are expected to have few or zero elements.
pub use smallvec::{SmallVec, smallvec};

/// A `ThingSet` is a hash set of `*const dyn Things`. They are useful if you need to test membership but never need
/// to access the original `Thing`.
pub type Set<T> = StdHashSet<T>; // This replaces Maude's `PointerSet` in most situations.
// endregion

// Logging
pub use tracing;
pub mod log;

pub use unsafe_ptr::UnsafePtr;


// Interned string. Use `DefaultAtom` for a global cache that can be used across threads. Use `Atom` for a thread-local
// string cache.
pub use string_cache::DefaultAtom as IString;
// pub use ustr::Ustr as IString;

// Heap construction/destruction
// pub use heap::{heap_construct, heap_destroy};

// region Items meant to be used only internally

pub use graph::Graph;

// A set of (small) natural numbers
pub use nat_set::NatSet;

// Reference counted pointers with mutable stable, and complementary weak pointers.
pub use rccell::{RcCell, WeakCell};

// Join sequences with a separator
pub use string_util::{join_string, join_iter, int_to_subscript};


pub use erased::DynHash;

// endregion
