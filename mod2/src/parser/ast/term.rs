use std::{
  cell::RefCell,
  collections::hash_map::Entry,
  rc::Rc,
};

use mod2_abs::{HashMap, IString, RcCell, rc_cell, heap_construct};
use mod2_lib::api::free_theory::FreeTerm;
use mod2_lib::api::symbol_core;
use mod2_lib::api::symbol_core::{Symbol, SymbolPtr};
use mod2_lib::api::term::{BxTerm, Term};
use crate::{
  builtin::{
    integer_symbol::IntegerSymbol,
    string_symbol::StringSymbol
  },
  NaturalNumber
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
  StringLiteral(String),
  NaturalNumber(NaturalNumber),
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
        let args = tail.into_iter().map(|t| Box::new(t.construct(symbols))).collect();
        term.args = args;

        Box::new(term)
      }

      TermAST::StringLiteral(string_literal) => {
        // ToDo: Where do we store literal symbols? They cannot be stored in the `symbols`  `HashMap` because they have
        //       no names.
        let symbol = heap_construct!(StringSymbol::new(string_literal.clone()));

        Term {
          term_node: TermNode::Symbol(symbol),
          attributes: TermAttributes::default()
        }
      }

      TermAST::NaturalNumber(natural_number) => {
        // ToDo: As with string literals, figure out if number literal symbols should be stored and reused.
        let symbol = heap_construct!(IntegerSymbol::new(natural_number.clone()));

        Term {
          term_node: TermNode::Symbol(symbol),
          attributes: TermAttributes::default()
        }
      }

    }
  }
}
