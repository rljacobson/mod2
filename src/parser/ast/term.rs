use std::{
  cell::RefCell,
  collections::hash_map::Entry,
  rc::Rc,
};

use crate::{
  abstractions::{
    HashMap,
    IString,
    RcCell,
    rc_cell,
    NaturalNumber
  },
  builtin::{
    integer_symbol::IntegerSymbol,
    string_symbol::StringSymbol
  },
  theory::{
    symbol::{
      RcSymbol,
      Symbol
    },
    term::{
      Term,
      TermAttributes,
      TermNode
    }
  }
};

pub(crate) type BxTermAST = Box<TermAST>;
pub(crate) enum TermAST {
  /// An identifier is a variable or symbol.
  Identifier(IString),

  /// Function Application: `head(tail).
  Application{
    head: BxTermAST,
    tail: Vec<BxTermAST>
  },

  // Literals are converted into symbols. See `symbol_type.rs`.
  StringLiteral(String),
  NaturalNumber(NaturalNumber),
}

impl TermAST {
  pub fn construct(&self, symbols: &mut HashMap<IString, RcSymbol>) -> Term {
    // ToDo: How do we construct term attributes.

    match self {

      TermAST::Identifier(name) => {
        let symbol = match symbols.entry(*name) {
          Entry::Occupied(s) => s.get().clone(),
          Entry::Vacant(v) => {
            let s = rc_cell!(Symbol::new(*name));
            v.insert(s.clone());
            s
          }
        };
        Term {
          term_node : TermNode::Symbol(symbol),
          attributes: TermAttributes::default()
        }
      }

      TermAST::Application { head, tail } => {

        Term {
          term_node: TermNode::Application {
            head: Box::new(head.construct(symbols)),
            tail: tail.into_iter().map(|t| Box::new(t.construct(symbols))).collect(),
          },
          attributes: TermAttributes::default()
        }
      }

      TermAST::StringLiteral(string_literal) => {
        // ToDo: Where do we store literal symbols? They cannot be stored in the `symbols`  `HashMap` because they have
        //       no names.
        let symbol = rc_cell!(StringSymbol::new(string_literal.clone()));

        Term {
          term_node: TermNode::Symbol(symbol),
          attributes: TermAttributes::default()
        }
      }

      TermAST::NaturalNumber(natural_number) => {
        // ToDo: As with string literals, figure out if number literal symbols should be stored and reused.
        let symbol = rc_cell!(IntegerSymbol::new(natural_number.clone()));

        Term {
          term_node: TermNode::Symbol(symbol),
          attributes: TermAttributes::default()
        }
      }

    }
  }
}
