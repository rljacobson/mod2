/*!

A sort is a type. Sorts can have other sorts as super sorts and sub sorts. The subsort relation `sort1 < sort2` defines
a lattice of sorts the connected components of which are called kinds.

Sorts are owned by the modules in which they are declared. Every other collection containing sorts holds weak references
to the sorts it contains.

*/

pub mod kind;
pub mod sort;
pub mod sort_spec;
pub mod collection;
mod kind_error;

pub use sort::*;
