/*!

Engine internals, including the GC, interpreter, algorithms, and internal data structures.

*/

#[cfg(feature = "profiling")]
mod profile;

mod local_bindings;
mod module;
mod narrowing_variable_info;
mod term_bag;
mod theory;
mod variable_info;
pub mod format;
pub mod pre_equation;
pub mod sort;
pub mod symbol;
pub mod term_core;
pub(crate) mod dag_node_core;
pub(crate) mod gc;
pub(crate) mod strategy;
pub(crate) mod substitution;
mod redex_position;
mod interpreter;

// Reexports to flatten some of the smaller modules
pub(crate) use local_bindings::LocalBindings;
pub(crate) use narrowing_variable_info::NarrowingVariableInfo;
pub(crate) use variable_info::VariableInfo;
pub(crate) use term_bag::TermBag;
// Public API
pub use module::*;
pub use theory::*;


#[allow(unused_imports)]
pub use gc::root_container::RootContainer;

/// A `*mut Void` is a pointer to a `u8`
// ToDo: Should this be `()`?
pub type Byte = u8;


#[cfg(test)]
mod tests {
  use mod2_abs::IString;
  use crate::{
    api::symbol::SymbolPtr,
    core::{
      dag_node_core::{
        DagNodeFlags,
        DagNodeCore
      },
      EquationalTheory,
      term_core::TermCore,
    },
  };

  #[test]
  fn size_of_types() {
    // A machine word is the size of usize.
    let word_size = size_of::<usize>();

    // Print header with right-justified columns.
    println!(
      "{:<16} {:>7} {:>7} {:>7}",
      "Type", "Words", "Bytes", "Bits"
    );
    // println!("{}", "─".repeat(16 + 10 + 10 + 4));

    // A helper macro to print the information for each type.
    macro_rules! print_size {
        ($type:ty) => {{
            let bytes = size_of::<$type>();
            // Round up bytes to nearest word.
            let words = (bytes + word_size - 1) / word_size;
            let bits = bytes * 8;
            println!("{:<16} {:>7} {:>7} {:>7}",
                     stringify!($type), words, bytes, bits);
        }};
    }

    print_size!(Vec<usize>);
    print_size!(String);
    print_size!(IString);
    print_size!(TermCore);
    print_size!(SymbolPtr);
    print_size!(EquationalTheory);
    print_size!(DagNodeFlags);
    print_size!(DagNodeCore);
  }
}
