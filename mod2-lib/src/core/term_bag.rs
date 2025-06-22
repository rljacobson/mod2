/*!

A `TermBag` is a cache of terms occurring in patterns or right hand sides that can be reused in building right hand
sides to enable common subexpression sharing both within rhs and lhs->rhs.

A special __structural_hash__ is used to determine if two terms are "the same." Sameness means only that a term from the
term bag can be used in place of the given term. In fact, even when two terms are semantically the same, there may be
reasons why a separate copy needs to be used instead of a shared cached term. In particular, if local changes need to be
made to a term without effecting all other semantically identical terms, then the globally shared cached term can't be
used.

We need a bimap between structural_hash and terms. The simplest way is to use the structural_hash as a key. Then

```ignore
term -> structural_hash  : just the structural_hash function itself
structural_hash -> term  : a lookup in the "term bag."
```

Maude uses "small" numbers in place of the structural_hash. The term bag must keep track of these numbers, and they are
not deterministic. It isn't clear to me why.

In Maude, a `TermBag` is a thin wrapper around a PointerSet.

*/

use std::{
  collections::hash_map::Entry,
  ops::Deref,
};
use mod2_abs::{
  HashMap,
  warning
};
use crate::{
  api::TermPtr,
  HashType
};


// region TermHashSet
/// For use with `TermHashSet` below.
type TermHashSet = HashMap<HashType, TermPtr>;

/// A trait extension of `HashMap<TermPtr>`. We use a `HashMap` so that we have access to the hash value (the key)
trait TermHashSetExt {
  /// Inserts the value into the set if it is not already present, returning `Option<existing_value>` if
  /// `existing_value` already exists in the map. An existing value is never replaced.
  fn insert_no_replace(&mut self, value: TermPtr) -> Option<TermPtr>;
  /// Fetches the value from the set, returning `None` if it is not present.
  fn find_for_hash(&self, hash: HashType) -> Option<TermPtr>;
  /// Finds the provided term, if it is in the set.
  fn find(&self, value: TermPtr) -> Option<(TermPtr, HashType)>;
  fn contains(&self, value: TermPtr) -> bool;
}

impl TermHashSetExt for HashMap<HashType, TermPtr> {
  fn insert_no_replace(&mut self, value: TermPtr) -> Option<TermPtr> {
    match self.entry(value.deref().structural_hash()) {
      Entry::Occupied(entry) => {
        Some(*entry.get())
      }
      Entry::Vacant(entry) => {
        entry.insert(value);
        None
      }
    }
  }

  fn find_for_hash(&self, hash: HashType) -> Option<TermPtr> {
    self.get(&hash).cloned()
  }

  fn find(&self, value: TermPtr) -> Option<(TermPtr, HashType)> {
    let key = value.deref().structural_hash();
    self.find_for_hash(key).map(|v| (v, key))

  }

  fn contains(&self, value: TermPtr) -> bool {
    self.find(value).is_some()
  }

}
// endregion TermHashSet


pub struct TermBag {
  terms_usable_in_eager_context: TermHashSet,
  terms_usable_in_lazy_context:  TermHashSet,
}

impl Default for TermBag {
  fn default() -> Self {
    TermBag {
      terms_usable_in_eager_context: TermHashSet::default(),
      terms_usable_in_lazy_context:  TermHashSet::default(),
    }
  }
}

impl TermBag {
  #[inline(always)]
  pub fn new() -> Self {
    Self::default()
  }

  /// Inserts the matched term if it is not already present in the `TermBag`. If it is already in the `TermBag`, no
  /// action is taken. In debug mode, attempting to insert an existing term results in an error.
  #[inline(always)]
  pub(crate) fn insert_matched_term(&mut self, term: TermPtr, eager_context: bool) {
    // New matched terms can never replace built terms (which are available at zero cost) nor existing matched terms
    // (for which the cost of storing the extra pointer may already have been paid).
    let success = self.terms_usable_in_lazy_context.insert_no_replace(term.clone()).is_none();
    debug_assert!(success, "TermBag should not insert a term that is already in the bag");
    if eager_context {
      let success = self.terms_usable_in_eager_context.insert_no_replace(term.clone()).is_none();
      debug_assert!(success, "TermBag should not insert a term that is already in the bag");
    }
  }

  /// Inserts a built term, replacing any existing term within the `TermBag`. New built terms should not arise if there
  /// is an existing usable term in the appropriate context, so a warning is emitted.
  #[inline(always)]
  pub(crate) fn insert_built_term(&mut self, term: TermPtr, eager_context: bool) {
    // New built terms should not arise if there is an existing usable term in the appropriate context.
    if eager_context {
      let success = self.terms_usable_in_eager_context.insert(term.structural_hash(), term).is_none();
      if !success {
        warning!(0, "re-insertion of {}", term);
      }
    } else {
      let success = self.terms_usable_in_lazy_context.insert(term.structural_hash(), term).is_none();
      if !success {
        warning!(0, "re-insertion of {}", term);
      }
    }
  }

  #[inline(always)]
  pub fn contains(&self, term: TermPtr, eager_context: bool) -> bool
  {
    if eager_context {
      self.terms_usable_in_eager_context.contains(term)
    } else {
      self.terms_usable_in_lazy_context.contains(term)
    }
  }

  /// Finds the provided term in the term bag, returning `None` if it is not present.
  #[inline(always)]
  pub fn find(&self, term: TermPtr, eager_context: bool) -> Option<(TermPtr, HashType)>
  {
    if eager_context {
      self.terms_usable_in_eager_context.find(term)
    } else {
      self.terms_usable_in_lazy_context.find(term)
    }
  }

  /// Fetches  the value from the set, returning `None` if it is not present.
  #[inline(always)]
  pub fn find_for_hash(&self, hash: HashType, eager_context: bool) -> Option<TermPtr> {
    if eager_context {
      self.terms_usable_in_eager_context.find_for_hash(hash)
    } else {
      self.terms_usable_in_lazy_context.find_for_hash(hash)
    }
  }
}
