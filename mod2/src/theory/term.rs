/*!

A term is a concrete instance of a constant symbol or application of a function.

Example: A function symbol `f` and constant symbol `x` can be used to form the term `f(f(x), x)`.
While there is only a single `Symbol` for `f`, there are two (sub)`Term`s in which `f` appears. Likewise for `x`.


*/

use std::{
  fmt::{Display, Formatter},
  rc::Rc
};

use enumflags2::{bitflags, BitFlags};
use mod2_abs::{join_iter, join_string, NatSet};

use crate::{
  theory::{
    symbol::{
      SymbolPtr,
      Symbol
    }
  }
};

/// A `Term` struct holds other subterms in its `term_node` field.
pub struct Term {
  pub term_node : TermNode,
  pub attributes: TermAttributes
}
pub type BxTerm = Box<Term>;

impl Term {
  pub fn true_literal() -> BxTerm {
    Box::new(Term{
      term_node : TermNode::Symbol(Symbol::true_literal()),
      attributes: TermAttributes::default()
    })
  }

  pub fn false_literal() -> BxTerm {
    Box::new(Term{
      term_node : TermNode::Symbol(Symbol::false_literal()),
      attributes: TermAttributes::default()
    })
  }
}

impl Display for Term {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.term_node)
  }
}

/// The part of the term that holds the subterms.
pub enum TermNode{

  Symbol(SymbolPtr),

  Application {
    head: BxTerm,
    tail: Vec<BxTerm>
  }

}

impl Display for TermNode {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      TermNode::Symbol(symbol) => {
        assert!(!symbol.is_null());
        write!(f, "{}", unsafe{&(**symbol).name})
      }
      TermNode::Application { head, tail } => {
        write!(f, "{}({})", head, join_string(tail.iter(), ", ") )
      }
    }
  }
}


#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TermAttribute {
  /// A subterm is stable if its top symbol cannot change under instantiation.
  Stable,
  /// A subterm is in an eager context if the path to its root contains only eagerly evaluated positions.
  EagerContext,
  /// A subterm "honors ground out match" if its matching algorithm guarantee never to return a matching subproblem
  /// when all the terms variables are already bound.
  HonorsGroundOutMatch
}
pub type TermAttributes = BitFlags<TermAttribute>;
