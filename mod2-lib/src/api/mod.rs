#![allow(unused_imports, dead_code)]
/*!

The public API of the library.

*/

use mod2_abs::optimizable_int::OptU16;

mod symbol;
mod term;
mod dag_node;
pub mod variable_theory;
pub mod free_theory;
pub mod built_in;
mod dag_node_cache;
mod automaton;
mod subproblem;
mod extension_info;

// Flatten hierarchy for non-theory submodules.
pub use symbol::*;
pub use term::*;
pub use dag_node::*;
pub use automaton::*;
pub use subproblem::*;
pub use extension_info::*;

pub type Arity = OptU16;
