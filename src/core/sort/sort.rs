use crate::abstractions::{IString, RcCell, WeakCell};
use crate::core::sort::kind::{Kind, WkKind};

// The pointers inside a sort to other sorts have to be weak pointers, because we expect there to be cycles.
pub type RcSort    = RcCell<Sort>;
pub type WeakSort  = WeakCell<Sort>;
pub type PtrSort   = *Sort;
pub type WeakSorts = Vec<WeakSort>;


#[derive(Clone)]
pub struct Sort {
  pub name:       IString, // a.k.a ID
  /// The `sort_index` is the index of the sort within its connected component.
  /// Used as `number_unresolved_supersorts` when computing supersorts.
  pub sort_index: i32,
  // pub fast_test:  i32,

  pub subsorts  : WeakSorts, // There may be sorts within the connected component that are
  pub supersorts: WeakSorts, // incomparable to this one and thus neither a super- nor sub-sort.
  // pub leq_sorts:  NatSet,

  // The connected component this sort belongs to.
  pub kind: * Kind, // This should be a weak reference
}

impl Default for Sort {
  fn default() -> Self {
    Sort {
      name: IString::default(),
      sort_index: 0,
      subsorts: WeakSorts::default(),
      supersorts: WeakSorts::default(),
      kind: std::ptr::null_mut(),
    }
  }
}

impl Sort {
  pub fn new(name: IString) -> Sort {
    Sort{
      name,
      ..Self::default()
    }
  }

  /// Returns -1 for special sort Any, -2 for special sort None, and 0 otherwise.
  pub fn arity(&self) -> i16 {
    match self.name  {
      v if v == IString::from("Any")  => -1,
      v if v == IString::from("None") => -2,
      _ =>  0
    }
  }
}
