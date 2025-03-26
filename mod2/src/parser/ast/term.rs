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
  }
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
}

impl TermAST {
  pub fn construct(&self, symbols: &mut HashMap<IString, SymbolPtr>) -> BxTerm {
    // ToDo: How do we construct term attributes.

    match self {

      TermAST::Identifier(name) => {
        let symbol = get_or_create_symbol(name, symbols);
        Box::new(FreeTerm::new(symbol))
      }

      TermAST::Application { name, tail } => {
        let symbol = get_or_create_symbol(name, symbols);

        let mut term = FreeTerm::new(symbol);
        let args = tail.into_iter().map(|t| t.construct(symbols)).collect();
        term.args = args;

        Box::new(term)
      }

      TermAST::StringLiteral(string_literal) => {
        Box::new(StringTerm::new(string_literal))
      }

      // TermAST::NaturalNumber(natural_number) => {
      //   Box::new(NaturalNumberTerm::new(natural_number))
      // }

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
