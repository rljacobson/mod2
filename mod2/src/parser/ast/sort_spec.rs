use mod2_abs::IString;
use mod2_lib::core::sort::{
  collection::SortCollection,
  sort_spec::{BxSortSpec, SortSpec}
};

pub(crate) type BxSortSpecAST = Box<SortSpecAST>;

/// Mirrors `crate::core::sort::SortSpec` but uses IString instead of `RcSort`.
pub(crate) enum SortSpecAST {
  Sort(IString),
  Functor{
    arg_sorts: Vec<BxSortSpecAST>,
    sort_spec: BxSortSpecAST
  }
}

impl SortSpecAST {
  /// Constructs the special `SortSpecAST` with sort name "Any".
  pub fn any() -> Box<SortSpecAST> {
    Box::new(SortSpecAST::Sort(IString::from("Any")))
  }

  /// Constructs the special `SortSpecAST` with sort name "None".
  pub fn none() -> Box<SortSpecAST> {
    Box::new(SortSpecAST::Sort(IString::from("None")))
  }

  /// Constructs a `SortSpec` from a `SortSpecAST` using sort objects from the given `SortCollection`.
  pub fn construct(&self, sorts: &mut SortCollection) -> BxSortSpec {
    match self {

      SortSpecAST::Sort(name) => {
        let sort = sorts.get_or_create_sort(name.clone());
        Box::new(SortSpec::Sort(sort))
      }

      SortSpecAST::Functor {
        arg_sorts, sort_spec
      } => {
        let constructed_arg_specs: Vec<BxSortSpec> =
            arg_sorts.iter()
                     .map(|spec_ast| spec_ast.construct(sorts))
                     .collect();
        let rhs_sort_spec = sort_spec.construct(sorts);
        Box::new(
          SortSpec::Functor {
            arg_sorts: constructed_arg_specs,
            sort_spec: rhs_sort_spec
          }
        )
      }

    }
  }

  pub fn arity(&mut self) -> i16 {
    match self {
      SortSpecAST::Sort(_) => 0,
      SortSpecAST::Functor { arg_sorts, .. } => arg_sorts.len() as i16
    }
  }
}
