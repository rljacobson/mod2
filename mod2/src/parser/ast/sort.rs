use mod2_abs::{smallvec, IString, SmallVec};
use mod2_lib::{
  api::{
    built_in::Integer,
    Arity
  },
  core::{
    sort::{
      collection::SortCollection,
      SortPtr,
    },
    symbol::TypeSignature
  }
};

pub(crate) type BxFunctorSortAST = Box<FunctorSortAST>;
pub(crate) type BxSortIdAST      = Box<SortIdAST>;
pub(crate) type BxSortSpecAST    = Box<SortSpecAST>;

pub(crate) enum SortSpecAST {
  Sort(BxSortIdAST),
  Functor(BxFunctorSortAST)
}

impl SortSpecAST {
  pub fn construct(&self, sorts: &mut SortCollection) -> TypeSignature {
    match self {
      SortSpecAST::Sort(sort_id) => {
        smallvec![sort_id.construct(sorts)]
      }
      SortSpecAST::Functor(functor) => {
        functor.construct(sorts)
      }
    }
  }
}

pub(crate) struct SortIdAST (pub IString);

impl SortIdAST {
  pub fn construct(&self, sorts: &mut SortCollection) -> SortPtr {
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
  pub(crate) arg_sorts  : Vec<BxSortIdAST>,
  pub(crate) target_sort: BxSortIdAST
}

impl FunctorSortAST {

  /// Constructs a `TypeSignature` from a `SortSpecAST` using sort objects from the given `SortCollection`.
  pub fn construct(&self, sorts: &mut SortCollection) -> TypeSignature {
    let mut constructed_arg_sorts: TypeSignature =
            self.arg_sorts
                .iter()
                .map(|sort_id| sort_id.construct(sorts))
                .collect();
    let rhs_sort = self.target_sort.construct(sorts);
    constructed_arg_sorts.push(rhs_sort);
    constructed_arg_sorts
  }

}
