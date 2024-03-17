use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::path::Iter;
use crate::abstractions::IString;
use crate::core::sort::RcSort;

/// A set of unique sorts with helper methods for creating new sorts. Helper collection only used during module construction.
#[derive(Default)]
pub struct SortCollection {
  sorts: HashMap<IString, RcSort>
}

impl SortCollection {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn get_or_create_sort(&mut self, name: IString) -> RcSort {
    match self.sorts.entry(name) {
      Entry::Occupied(s) => s.get().clone(),
      Entry::Vacant(v) => {
        let s = rc_cell!(Sort::new(name));
        v.insert(s.clone());
        s
      }
    }
  }

  /// Given a list of sort names, inserts or creates a sort for each name.
  pub fn create_implicit_sorts(&mut self, sort_names: &mut HashSet<IString>) {
    for sort_name in sort_names.drain() {
      self.get_or_create_sort(sort_name);
    }
  }

  /// Creates and returns an iterator over the `SortCollection`.
  fn iter(&self) -> Iter {
    self.sorts.iter().map(|istr, rcs| (istr.clone(), rcs.clone()))
  }
}
