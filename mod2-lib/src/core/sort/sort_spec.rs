/*!

A [`SortSpec`](crate::core::sort::sort_spec::SortSpec) is a generalization of `Sort` that additionally permits
functors. `SortSpec`s are not named.

## See Also...

 - A [`Sort`](crate::core::sort::sort::Sort) is a named type.
 - A [`Kind`](crate::core::sort::kind::Kind) is a connected component of the lattice of `Sort`s induced by the subsort
   relation.

*/

use std::fmt::Display;
use mod2_abs::join_string;
use crate::{
  core::sort::SortPtr,
  api::Arity
};

/// A boxed `SortSpec`.
pub type BxSortSpec = Box<SortSpec>;

/// A generalization of a `Sort` that additionally permits functors.
#[derive(Eq, PartialEq, Default)]
pub enum SortSpec {
  Sort(SortPtr),
  // arg1_sort arg2_sort -> target_sort
  Functor{
    arg_sorts: Vec<SortPtr>,
    target_sort: SortPtr
  },
  Any,  // Shortcut for `SortSpec::Sort(Rc::new(Sort::any()))`
  #[default]
  None, // Shortcut for `SortSpec::Sort(Rc::new(Sort::none()))`
}

impl SortSpec {
  pub fn arity(&self) -> Arity {
    match self {

      SortSpec::Sort(sort) => {
        sort.arity()
      },

      SortSpec::Functor { arg_sorts, ..} => Arity::Value(arg_sorts.len() as u16),

      SortSpec::Any => Arity::Any,

      SortSpec::None => Arity::Unspecified

    }
  }
}


impl Display for SortSpec {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {

      SortSpec::Sort(sort) => {
        write!(f, "{}", sort)
      }

      SortSpec::Functor { arg_sorts, target_sort} => {
        let arg_string = join_string(arg_sorts.iter(), " ");
        write!(f, "{} -> {}", arg_string, target_sort)
      }

      SortSpec::Any => {
        write!(f, "Any")
      }

      SortSpec::None => {
        write!(f, "none")
      }

    }
  }
}
