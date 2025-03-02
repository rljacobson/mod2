# mod2-abs - Abstractions

Types/type aliases that abstract over the implementing backing type.

# Utilities

- Heap Memory Management
  - **`heap_construct!`:** Creates a heap-allocated object and returns a raw pointer (`*mut T`) to it, bypassing Rust's automatic memory management. The user takes responsibility for manually freeing the memory.

  - **`heap_destroy!`:** Reclaims the memory associated with a raw pointer returned by `heap_construct!`. It converts the raw pointer back into a `Box<T>`, which is then dropped allowing Rust to deallocate the memory.

- **`IString`:** Interned string.

- **`log::*`:** Logging macros with optional thresholds (in addition to levels)
  - **`critical!`, `error!`, `warning!`, `info!`, `debug!`, `trace!`:** Macros for emitting log messages.
  - **`set_global_logging_threshold`, `get_global_logging_threshold`:** Sets and retrieves the threshold of the global logging filter.

- **`NatSet`:** A set of (small) natural numbers implemented as a thin wrapper around BitSet (the bit-set crate).

- **`Graph`:** Conflict graph resolved using a naive graph coloring algorithm.

- **`rccell::*`:** Reference counted pointers with mutable stable, and complementary weak pointers.

- **`string_util::*`**
  - **`join_string(iter, sep: &str)`:** Join a list of things that can be displayed as string with a given separator.
  - **`int_to_subscript(value: u32)`:** Converts a number to a unicode string representation of the number in subscript.
  - **`pub fn join_iter<T>(mut iter: impl Iterator<Item = T>, sep: impl Fn(&T) -> T) -> impl Iterator<Item = T>`:** Produces an iterator that riffles the given iterator with values computed from a function.

- **`DynHash`:** Implements the [erased trait](https://quinedot.github.io/rust-learning/dyn-trait-erased.html) pattern
  from [Learning Rust: Hashable Box<dyn Trait>](https://quinedot.github.io/rust-learning/dyn-trait-hash.html). So far we just do this to implement `Hash`.


## Background and Motivation

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

# Authorship and License


Copyright © 2025 Robert Jacobson. This software is distributed under the terms of the
[MIT license](LICENSE-MIT) or the [Apache 2.0 license](LICENSE-APACHE) at your preference.

This work contains code adapted with improvements from [rccell](https://crates.io/crates/rccell)
([GitHub](https://github.com/romancardenas/rccell)), which is Copyright © 2021 Román Cárdenas and distributed
under the MIT License.
