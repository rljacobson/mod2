/*!

A [`SortSpec`] is a generalization of `Sort` that additionally permits
functors. `SortSpec`s are not named.

## See Also...

 - A [`Sort`] is a named type.
 - A [`Kind`](crate::core::sort::kind::Kind) is a connected component of the lattice of `Sort`s induced by the subsort
   relation.

*/

use std::fmt::Display;
use mod2_abs::join_string;
use crate::{
  core::sort::{
    Sort,
    SortPtr
  },
  theory::symbol::UNSPECIFIED
};

/// A boxed `SortSpec`.
pub type BxSortSpec = Box<SortSpec>;

/// A generalization of a `Sort` that additionally permits functors.
pub enum SortSpec {
  Sort(SortPtr),
  // arg1_sort arg2_sort -> result_sort
  Functor{
    arg_sorts: Vec<BxSortSpec>,
    sort_spec: BxSortSpec
  },
  Any,  // Shortcut for `SortSpec::Sort(Rc::new(Sort::any()))`
  None, // Shortcut for `SortSpec::Sort(Rc::new(Sort::none()))`
}

impl SortSpec {
  pub fn arity(&self) -> i16 {
    match self {

      SortSpec::Sort(sort) => {
        assert!(!sort.is_null());
        unsafe {
          (**sort).arity()
        }
      },

      SortSpec::Functor { arg_sorts, ..} => arg_sorts.len() as i16,

      _ => UNSPECIFIED
    }
  }
}


impl Display for SortSpec {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {

      SortSpec::Sort(sort) => {
        assert!(!sort.is_null());
        write!(f, "{}", unsafe{ &*(*sort) })
      }

      SortSpec::Functor { arg_sorts, sort_spec } => {
        write!(f, "{} -> {}", join_string(arg_sorts.iter(), " "), sort_spec)
      }

      SortSpec::Any => {
        write!(f, "any")
      }

      SortSpec::None => {
        write!(f, "none")
      }

    }
  }
}
