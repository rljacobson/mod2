/*!

The `DagNode` is the heart of the engine. Speed hinges on efficient management of `DagNode` objects. Their creation,
reuse, and destruction are managed by an arena based garbage collecting allocator which relies on the fact that
every `DagNode` is of the same size. Since `DagNode`s can be of different types and have arguments, we make careful use
of transmute and bitflags.

The following compares Maude's `DagNode` to our implementation here.

|                | Maude                                        | mod2lib                     |
|:---------------|:---------------------------------------------|:----------------------------|
| size           | Fixed 3 word size (or 6 words?)              | Fixed size struct (4 words) |
| tag            | implicit via vtable pointer                  | enum variant                |
| flags          | `MemoryInfo` in first word                   | `BitFlags` field            |
| shared impl    | base class impl                              | enum impl                   |
| specialization | virtual function calls                       | match on variant in impl    |
| args           | `reinterpret_cast` of 2nd word based on flag | Nested enum                 |

*/

mod local_bindings;
mod narrowing_variable_info;
mod theory;
mod variable_info;
pub(crate) mod dag_node_core;
pub(crate) mod gc;
pub(crate) mod strategy;
pub(crate) mod substitution;
pub mod format;
pub mod pre_equation;
pub mod sort;
pub mod symbol;
pub mod term_core;

// Reexports to flatten some of the smaller modules
pub(crate) use local_bindings::LocalBindings;
pub(crate) use narrowing_variable_info::NarrowingVariableInfo;
pub(crate) use variable_info::VariableInfo;
pub use theory::*;


#[allow(unused_imports)]
pub use gc::root_container::RootContainer;

/// A `*mut Void` is a pointer to a `u8`
pub type Void = u8;


#[cfg(test)]
mod tests {
  use crate::{
    api::symbol::SymbolPtr,
    core::{
      dag_node_core::{
        DagNodeFlags,
        DagNodeCore
      },
      EquationalTheory
    },
  };

  #[test]
  fn size_of_dag_node() {
    println!("size of SymbolPtr: {}", size_of::<SymbolPtr>());
    println!("size of EquationalTheory: {}", size_of::<EquationalTheory>());
    println!("size of DagNodeFlags: {}", size_of::<DagNodeFlags>());
    println!("size of DagNode: {}", size_of::<DagNodeCore>());
    assert_eq!(size_of::<DagNodeCore>(), 4 * size_of::<usize>());
  }
}
