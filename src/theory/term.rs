/*!

A term is a concrete instance of a constant symbol or application of a function.

Example: A function symbol `f` and constant symbol `x` can be used to form the term `f(f(x), x)`.
While there is only a single `Symbol` for `f`, there are two (sub)`Term`s in which `f` appears. Likewise for `x`.


*/

use std::rc::Rc;

use enumflags2::{bitflags, BitFlags};

use crate::{
  abstractions::NatSet,
  theory::{
    symbol::{
      RcSymbol,
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

/// The part of the term that holds the subterms.
pub enum TermNode{

  Symbol(RcSymbol),

  Application {
    head: BxTerm,
    tail: Vec<BxTerm>
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
