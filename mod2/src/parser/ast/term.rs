use std::{
  cell::RefCell,
  collections::hash_map::Entry,
  rc::Rc,
};

use mod2_abs::{HashMap, IString, RcCell, rc_cell, heap_construct};
use mod2_lib::{
  api::{
    symbol::{Symbol, SymbolPtr},
    free_theory::FreeTerm,
    built_in::{
      Bool,
      BoolTerm,
      Float,
      FloatTerm,
      Integer,
      IntegerTerm,
      NaturalNumber,
      NaturalNumberTerm,
      StringBuiltIn,
      StringTerm,
      get_built_in_symbol,
    },
    term::{BxTerm, Term},
  },
  core::sort::SortCollection
};
use crate::parser::ast::get_or_create_symbol;

pub(crate) type BxTermAST = Box<TermAST>;
pub(crate) enum TermAST {
  /// An identifier is a variable or symbol.
  Identifier(IString),

  /// Function Application: `head(tail).
  Application{
    name: IString,
    tail: Vec<BxTermAST>
  },

  // Literals are converted into symbols. See `symbol_type.rs`.
  StringLiteral(StringBuiltIn),
  NaturalNumber(NaturalNumber),
  Integer(Integer),
  Float(Float),
  Bool(Bool)
}

impl TermAST {
  pub fn construct(&self, symbols: &mut HashMap<IString, SymbolPtr>, sorts: &mut SortCollection, variables: &mut HashMap<IString, BxTerm>) -> BxTerm {
    // ToDo: How do we construct term attributes.

    match self {

      TermAST::Identifier(name) => {
        if let Some(var_term) = variables.get(name) {
          return var_term.deep_copy();
        }
        let symbol = get_or_create_symbol(name, symbols);
        symbol.make_term(vec![])
      }

      TermAST::Application { name, tail } => {
        let symbol = get_or_create_symbol(name, symbols);
        let args   = tail.into_iter().map(|t| t.construct(symbols, sorts, variables)).collect();
        symbol.make_term(args)
      }

      TermAST::StringLiteral(string_literal) => {
        Box::new(StringTerm::new(string_literal))
      }

      TermAST::Bool(boolean) => {
        Box::new(BoolTerm::new(*boolean))
      }

      TermAST::Integer(integer) => {
        Box::new(IntegerTerm::new(*integer))
      }

      TermAST::Float(float) => {
        Box::new(FloatTerm::new(*float))
      }

      TermAST::NaturalNumber(natural_number) => {
        Box::new(NaturalNumberTerm::new(*natural_number))
      }
    }
  }
}


#[cfg(test)]
mod tests {
  use crate::parser::parser::TermParser;
  use super::*;

  #[test]
  fn test_construct() {
    let mut parser = TermParser::default();
    let input = "f(\"hello\", 1, 2.0, true, g(x, y, z, false))";
    let term = parser.parse(input).unwrap();

    let mut symbols = HashMap::new();
    let mut sorts = SortCollection::new();
    let mut variables = HashMap::new();

    let term = term.construct(&mut symbols, &mut sorts, &mut variables);
    println!("{:?}", term);
  }
}
