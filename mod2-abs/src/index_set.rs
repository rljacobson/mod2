/*!

An `IndexSet` swaps a value for an index (a `usize`) using some key value. We have two use cases:
1. When the value and key are the same, namely a pointer
2. When the key is provided by the client code, typically a structural_hash of the value.

In the first case, we provide the `IndexSet::insert` and `IndexSet::value_to_index` methods. In the second case,
we provide the `IndexSet::insert_with_hash` method.

This data structure replaces `PointerSet`, `DagNodeSet`, and `IndexedSet`.

Eker describes this as an important optimization. In Maude, `PointerSet` has two distinct use cases:
1. to assign a unique index to each pointer
2. to assign a unique index to each pointer with a provided hash value

In the first case, the hash value is an implementation detail that need not be exposed to the user. In the second case,
the hash value is a structural_hash, and we are associating both a canonical object and an index to this hash. In both
cases, the functionality of the `PointerSet` can be based on Maude's `IndexedSet` (but isn't), so we just implement
a single container for all cases.

ToDo: `PointerSet` stores a value upon insert in the case that the provided hash is equal to an existing hash but the
      pointer is different. This is only relevant with structural_hashes. It's not clear if this behavior is important.
      This implementation neither replaces the value nor stores the given value if the hash exists.

*/

use std::collections::hash_map::Entry;
use std::hash::Hash;
use crate::{HashMap, UnsafePtr};

pub struct IndexSet<Key, Value>
    where Key  : Hash + Clone + Eq, Value : PartialEq
{
  /// Maps an index to a key of the hash map below
  keys: Vec<Key>,
  // ToDo: Use a faster hasher
  /// Maps a key to an object and index. Sometimes the key is the hash value, and sometimes it is the pointer itself.
  indices: HashMap<Key, (usize, Value)>,
}

impl<Key, Value> Default for IndexSet<Key, Value>
    where Key  : Hash + Clone + Eq, Value : PartialEq
{
  fn default() -> Self {
    Self {
      keys: Vec::new(),
      indices: HashMap::new(),
    }
  }
}

// When the `Key` is the hash of `Value`
impl<Value> IndexSet<Value, Value>
where Value  : Hash + Clone + Eq
{

  /// Use the value's own hash to insert the pointer, returning the index
  pub fn insert(&mut self, value: Value) -> usize {
    let index = self.indices.len();

    match self.indices.entry(value.clone()) {

      Entry::Occupied(entry) => {
        // return just the index
        (*entry.get()).0
      }

      Entry::Vacant(entry) => {
        entry.insert((index, value.clone()));
        self.keys.push(value);
        index
      }

    }
  }

  /// Use the value's own hash to get the index.
  pub fn value_to_index(&self, value: Value) -> Option<usize> {
    self.indices.get(&value).map(| (index, _) | *index )
  }

}

// The more general case where the user provides the key. This subsumes the above case.
impl<Key, Value> IndexSet<Key, Value>
where Key  : Hash + Clone + Eq,
     Value : PartialEq
{
  pub fn new() -> Self {
    Self::default()
  }

  /// Insert the pointer with the given key
  pub fn insert_with_key(&mut self, key: Key, value: Value) -> usize {
    // We snatch this length here to avoid borrowing `self` twice.
    let index = self.indices.len();

    match self.indices.entry(key.clone()) {

      Entry::Occupied(entry) => {
        let (index, _) = *entry.get();
        index
      }

      Entry::Vacant(entry) => {
        entry.insert((index, value));
        self.keys.push(key);
        index
      }
    }
  }

  /// Use the given key to get the index. If the value found for the key is not equal to the given value, or if the key
  /// does not exist, returns `None`.
  pub fn key_to_index(&self, key: Key, value: Value) -> Option<usize> {
    self.indices
        .get(&key)
        .filter(|(_, v)| *v == value)
        .map(|(idx, _)| *idx)
  }


  /// Use the index to get the key
  pub fn index_to_key(&self, index: usize) -> Option<Key> {
    self.keys.get(index).cloned()
  }

  /// Use the index to get the value
  pub fn index_to_value(&self, index: usize) -> Option<&Value> {
    self.keys.get(index).map(
      // Guaranteed to exist, since the key is in the keys vector
      | key | &self.indices.get(key).unwrap().1
    )
  }


  pub fn len(&self) -> usize {
    self.keys.len()
  }

  pub fn clear(&mut self) {
    self.keys.clear();
    self.indices.clear();
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_and_default_impls() {
    let iset: IndexSet<u8, u8> = IndexSet::new();
    assert_eq!(iset.len(), 0);
    // Nothing to look up
    assert_eq!(iset.index_to_key(0), None);
    assert_eq!(iset.index_to_value(0), None);
    assert_eq!(iset.key_to_index(0, 0), None);

    let iset: IndexSet<u8, u8> = IndexSet::default();
    assert_eq!(iset.len(), 0);
    // Nothing to look up
    assert_eq!(iset.index_to_key(0), None);
    assert_eq!(iset.index_to_value(0), None);
    assert_eq!(iset.key_to_index(0, 0), None);
  }

  #[test]
  fn insert_with_key_basic() {
    let mut iset = IndexSet::new();
    let idx1 = iset.insert_with_key(10u8, "ten");
    assert_eq!(idx1, 0);
    assert_eq!(iset.len(), 1);

    // key → index
    assert_eq!(iset.key_to_index(10, "ten"), Some(0));

    // index → key/value
    assert_eq!(iset.index_to_key(0), Some(10));
    assert_eq!(iset.index_to_value(0), Some(&"ten"));

    // mismatched pointer or key yields None
    assert_eq!(iset.key_to_index(10, "wrong"), None);
    assert_eq!(iset.key_to_index(99, "ten"), None);
  }

  #[test]
  fn insert_with_hash_duplicates() {
    let mut index_set = IndexSet::new();
    let idx1 = index_set.insert_with_key(7u8, "seven");
    // inserting the same key again should return the same index without replacing the value
    let idx2 = index_set.insert_with_key(7, "SEVEN");
    assert_eq!(idx2, idx1);
    // and length should not grow
    assert_eq!(index_set.len(), 1);
    // value should be the original
    assert_eq!(index_set.index_to_value(idx1), Some(&"seven"));
  }

  #[test]
  fn clear_empties() {
    let mut iset = IndexSet::new();
    iset.insert_with_key(1, "a");
    iset.insert_with_key(2, "b");
    assert_eq!(iset.len(), 2);

    iset.clear();
    assert_eq!(iset.len(), 0);
    assert_eq!(iset.index_to_key(0), None);
    assert_eq!(iset.index_to_value(1), None);
  }

  // Now test the specialized Value=Key case
  #[test]
  fn specialized_insert_and_lookup() {
    let mut iset = IndexSet::<&'static str, &'static str>::new();
    // insert(value) is equivalent to insert_with_hash(value.clone(), value)
    let idx1 = iset.insert("apple");
    assert_eq!(idx1, 0);
    assert_eq!(iset.len(), 1);

    // value → index
    assert_eq!(iset.value_to_index("apple"), Some(0));
    assert_eq!(iset.value_to_index("banana"), None);

    // duplicate insert returns same index and updates nothing
    let idx2 = iset.insert("apple");
    assert_eq!(idx2, idx1);
    assert_eq!(iset.len(), 1);
  }

  #[test]
  fn mixed_key_types_work() {
    #[derive(Hash, Clone, Eq, PartialEq, Debug)]
    struct Key(u16);

    #[derive(Hash, Clone, Eq, PartialEq, Debug)]
    struct Val(&'static str);

    let mut index_set = IndexSet::<Key, Val>::new();
    let k1 = Key(100);
    let v1 = Val("hello");

    let idx = index_set.insert_with_key(k1.clone(), v1.clone());
    assert_eq!(index_set.len(), 1);
    assert_eq!(index_set.key_to_index(k1.clone(), v1.clone()), Some(idx));
    assert_eq!(index_set.index_to_key(idx), Some(k1));
    assert_eq!(index_set.index_to_value(idx), Some(&v1));
  }
}
