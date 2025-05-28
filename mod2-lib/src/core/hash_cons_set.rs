/*!

A `HashConsSet` implements structural sharing of subnodes in a DAG.

The primary purpose of `HashConsSet` is to ensure that structurally identical DAG nodes are
represented by a single canonical instance in memory. This

1. reduces memory usage by eliminating duplicate representations;
2. allows structural equality checks to be performed by simple pointer comparison;
3. provides a foundation for memoization of operations on terms.

*/

use crate::core::gc::root_container::RootSet;


pub struct HashConsSet {
  node_set: RootSet
}

impl HashConsSet {
  
}
