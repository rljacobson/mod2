/*!

A [`SortSpec`](crate::core::sort::sort_spec::SortSpec) is a generalization of `Sort` that additionally permits
functors. `SortSpec`s are not named.

## See Also...

 - A [`Sort`](crate::core::sort::sort::Sort) is a named type.
 - A [`Kind`](crate::core::sort::kind::Kind) is a connected component of the lattice of `Sort`s induced by the subsort
   relation.

*/

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
