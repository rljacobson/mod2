use std::{
  cell::RefCell,
  cmp::max,
  collections::hash_map::Entry,
  rc::Rc,
};

use crate::{abstractions::{
  HashMap,
  Integer,
  IString,
  rc_cell,
  RcCell
}, heap_construct, parser::ast::{
  attribute::AttributeAST,
  BxSortSpecAST
}, theory::{
  symbol::{
    SymbolPtr,
    Symbol,
    symbol_for_symbol_type
  },
  symbol_type::{
    CoreSymbolType,
    SymbolType
  },
}};
use crate::core::sort::collection::SortCollection;

pub(crate) type BxSymbolDeclarationAST = Box<SymbolDeclarationAST>;

pub(crate) struct SymbolDeclarationAST {
  pub name      : IString,
  pub attributes: Vec<AttributeAST>,
  pub arity     : Integer,               // -1 means variadic
  pub sort_spec : Option<BxSortSpecAST>, // Empty is the special "None" sort.
}

pub(crate) type BxVariableDeclarationAST = Box<VariableDeclarationAST>;

pub(crate) struct VariableDeclarationAST {
  pub name      : IString,
  pub attributes: Vec<AttributeAST>,
  pub arity     : Integer,               // -1 means variadic, -2 means unspecified
  pub sort_spec : Option<BxSortSpecAST>, // Empty is the special "Any" sort
}


/// Common code for VariableDeclarationAST and SymbolDeclarationAST
pub fn construct_symbol_from_decl(
  symbols         : &mut HashMap<IString, SymbolPtr>,
  sorts           : &mut SortCollection,
  name            : IString,
  sort_spec       : Option<BxSortSpecAST>,
  arity           : i16,
  attributes_ast  : Vec<AttributeAST>,
  core_symbol_type: CoreSymbolType,
)
{
  let sort_spec = sort_spec.map(|s| s.construct(sorts));
  // If an explicit arity is given, use it.
  let arity = match &sort_spec {
    None => arity,
    Some(sort_spec) => {
      max(arity, sort_spec.arity())
    }
  };

  // Construct the symbol type.
  let attributes  = AttributeAST::construct_attributes(&attributes_ast);
  let symbol_type = SymbolType {
    core_type: core_symbol_type,
    attributes,
  };
  let theory_symbol = symbol_for_symbol_type(&symbol_type);

  match symbols.entry(name) {

    Entry::Occupied(s) => {
      // ToDo: Under what circumstances would a symbol already exist? If the symbol is already declared, this
      //       should be a duplicate declaration and thus an error.
      panic!("duplicate symbol declaration")
      // let mut symbol       = s.get().borrow_mut();
      // symbol.arity         = arity;
      // symbol.sort_spec     = sort_spec;
      // symbol.symbol_type   = symbol_type;
      // symbol.theory_symbol = Some(theory_symbol);
    },

    Entry::Vacant(v) => {
      // The symbol doesn't exist. Create it.
      let mut s = heap_construct!(
            Symbol{
              name,
              arity,
              symbol_type,
              sort_spec,
              theory_symbol: Some(theory_symbol),
            }
          );
      v.insert(s);
    }

  };
}
