use crate::core::sort::RcSort;
use crate::theory::symbol::UNSPECIFIED;

pub type BxSortSpec = Box<SortSpec>;

pub enum SortSpec {
  Sort(RcSort),
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
        sort.borrow().arity()
      },

      SortSpec::Functor { arg_sorts, ..} => arg_sorts.len() as i16,

      _ => UNSPECIFIED
    }
  }
}
