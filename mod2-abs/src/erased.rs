/*!

Implements the [erased trait](https://quinedot.github.io/rust-learning/dyn-trait-erased.html) pattern
from [Learning Rust: Hashable Box<dyn Trait>](https://quinedot.github.io/rust-learning/dyn-trait-hash.html).

While this code is very generic, it isn't needed for users of the library. It only exists to support internal code.

So far we just do this to implement `Hash`.

To use `DynHash`, just implement `Hash` for your trait.

```rust
# use mod2lib::abstractions::DynHash;
use core::hash::{Hash, Hasher};

pub trait Trait: DynHash{}

impl Hash for dyn Trait {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state)
    }
}
```

*/
use core::hash::{Hash, Hasher};

pub trait DynHash {
  fn dyn_hash(&self, state: &mut dyn Hasher);
}

// impl<T: ?Sized + Hash> DynHash for T {
impl<T: Hash> DynHash for T {
  fn dyn_hash(&self, mut state: &mut dyn Hasher) {
    self.hash(&mut state)
  }
}

impl Hash for dyn DynHash + '_ {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.dyn_hash(state)
  }
}
