use mod2_abs::IString;
use mod2_lib::{
  core::{
    sort::{
      collection::SortCollection,
      SortPtr,
      sort_spec::{BxSortSpec, SortSpec}
    }
  }
};

pub(crate) type BxFunctorSortAST = Box<FunctorSortAST>;
pub(crate) type BxSortIdAST = Box<SortIdAST>;
pub(crate) type BxSortSpecAST = Box<SortSpecAST>;

pub(crate) enum SortSpecAST {
  Sort(BxSortIdAST),
  Functor(BxFunctorSortAST)
}

pub(crate) struct SortIdAST (pub IString);

impl SortIdAST {
  pub fn construct_sort_spec(&self, sorts: &mut SortCollection) -> BxSortSpec {
    let sort = self.construct_sort_ptr(sorts);
    Box::new(SortSpec::Sort(sort))
  }

  pub fn construct_sort_ptr(&self, sorts: &mut SortCollection) -> SortPtr {
    sorts.get_or_create_sort(self.0.clone())
  }

  /// Constructs the special `SortSpecAST` with sort name "Any".
  pub fn any() -> BxSortIdAST {
    SortIdAST::from_name(IString::from("Any"))
  }

  /// Constructs the special `SortSpecAST` with sort name "None".
  pub fn none() -> BxSortIdAST {
    SortIdAST::from_name(IString::from("None"))
  }

  pub fn from_name(name: IString) -> BxSortIdAST {
    Box::new(SortIdAST(name))
  }
}

pub(crate) struct FunctorSortAST {
  arg_sorts: Vec<BxSortIdAST>,
  target_sort: BxSortIdAST
}

impl FunctorSortAST {

  /// Constructs a `SortSpec` from a `SortSpecAST` using sort objects from the given `SortCollection`.
  pub fn construct(&self, sorts: &mut SortCollection) -> BxSortSpec {
    let constructed_arg_sorts: Vec<SortPtr> =
            self.arg_sorts
                .iter()
                .map(|sort_id| sort_id.construct_sort_ptr(sorts))
                .collect();
    let rhs_sort = self.target_sort.construct_sort_ptr(sorts);
    Box::new(
      SortSpec::Functor {
        arg_sorts: constructed_arg_sorts,
        target_sort: rhs_sort
      }
    )
  }

  pub fn arity(&mut self) -> i16 {
    self.arg_sorts.len() as i16
  }
}
