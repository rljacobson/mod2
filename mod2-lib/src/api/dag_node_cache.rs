/*!

A small utility container to allow structural sharing during dagification.

In Maude, the global cache is cleared at the start of conversion of an expression tree. We pass around a reference to 
a new cache instead.

Maude also stores a pointer to the term itself. 

*/

use std::collections::HashMap;
use crate::{
  api::{
    dag_node::{DagNode, DagNodePtr},
    term::Term
  },
  HashType
};

#[derive(Default)]
pub(crate) struct DagNodeCache {
  pub set_sort_info: bool,
  pub map: HashMap<HashType, DagNodePtr>,
}

impl DagNodeCache {
  pub fn new(set_sort_info: bool) -> Self {
    DagNodeCache{
      set_sort_info,
      ..DagNodeCache::default()
    }
  }
  
  // pub fn get(&self, term: &dyn Term) -> Option<DagNodePtr> {
  //   self.map.get(&term.hash()).copied()
  // }
  
  #[inline(always)]
  pub fn get(&self, hash: HashType) -> Option<DagNodePtr> {
    self.map.get(&hash).copied()
  }
  
  // pub fn insert(&mut self, term: &dyn Term, node: DagNodePtr) {
  //   self.map.insert(term.hash(), node);
  // }

  #[inline(always)]
  pub fn insert(&mut self, hash: HashType, node: DagNodePtr) {
    self.map.insert(hash, node);
  }
}