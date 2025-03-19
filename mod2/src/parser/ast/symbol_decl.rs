use std::{
  cell::RefCell,
  cmp::max,
  collections::hash_map::Entry,
  rc::Rc,
};

use mod2_abs::{
  HashMap,
  IString,
  rc_cell,
  RcCell,
  heap_construct,
};
use mod2_lib::{
  api::{
    Arity,
    symbol::{
      SymbolPtr,
      Symbol,
    },
    built_in::{
      get_built_in_sort, 
      Integer, 
      StringBuiltIn, 
      Float, 
      NaturalNumber, 
      Bool
    }
  },
  core::sort::collection::SortCollection,
};
use mod2_lib::core::symbol_core::SymbolType;
use crate::{
  parser::{
    ast::{
      attribute::AttributeAST,
      BxFunctorSortAST,
      BxSortIdAST
    }
  },
};

pub(crate) type BxSymbolDeclarationAST = Box<SymbolDeclarationAST>;

pub(crate) struct SymbolDeclarationAST {
  pub name      : IString,
  pub attributes: Vec<AttributeAST>,
  pub arity     : Integer,               // -1 means variadic, -2 means unspecified
  pub sort_spec : Option<BxFunctorSortAST>, // Empty is the special "None" sort.
}

pub(crate) type BxVariableDeclarationAST = Box<VariableDeclarationAST>;

pub(crate) struct VariableDeclarationAST {
  pub name      : IString,
  pub attributes: Vec<AttributeAST>,
  pub sort      : Option<BxSortIdAST>, // Empty is the special "Any" sort
}


/// Common code for VariableDeclarationAST and SymbolDeclarationAST
pub fn construct_symbol_from_decl(
  symbols         : &mut HashMap<IString, SymbolPtr>,
  sorts           : &mut SortCollection,
  name            : IString,
  sort_spec       : Option<BxSortIdAST>,
  attributes_ast  : Vec<AttributeAST>,
  symbol_type     : SymbolType,
)
{
  let sort_spec = sort_spec.map(|s| s.construct(sorts));
  // If an explicit arity is given, use it.
  let arity = match &sort_spec {
    None => 0,
    Some(sort_spec) => {
      sort_spec.arity().into()
    }
  };

  // Construct the symbol type.
  let attributes = AttributeAST::construct_attributes(&attributes_ast);


  match symbols.entry(name.clone()) {

    Entry::Occupied(s) => {
      // ToDo: Under what circumstances would a symbol already exist? If the symbol is already declared, this
      //       should be a duplicate declaration and thus an error.
      panic!("duplicate symbol declaration");
    },

    Entry::Vacant(v) => {
      // The symbol doesn't exist. Create it.
      let mut s = Symbol::new(
        name,
        Arity::from(arity),
        attributes,
        symbol_type,
        unsafe{ get_built_in_sort("Any").unwrap_unchecked() },
      );

      v.insert(s);
    }

  };
}
