#![allow(unused_imports, dead_code)]
/*!

The public API of the library.

*/

use mod2_abs::optimizable_int::OptU16;

pub mod symbol;
pub mod term;
pub(crate) mod dag_node;
pub mod variable_theory;
pub mod free_theory;
pub mod built_in;
mod dag_node_cache;
pub mod automaton;
pub mod subproblem;

pub(crate) type ArgIndex = u16;
pub type Arity = OptU16;
