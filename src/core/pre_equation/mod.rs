/*!

A `PreEquation` is just a superclass for equations, rules, sort constraints, and strategies (the last of which is not
implemented.) The subclass is implemented as enum `PreEquationKind`.

*/

pub mod condition;

use enumflags2::{bitflags, BitFlags};

use crate::{
  abstractions::IString,
  theory::term::BxTerm
};
use crate::core::pre_equation::condition::Conditions;
use crate::core::sort::sort::RcSort;


#[bitflags]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PreEquationAttribute {
  Compiled,     // PreEquation
  NonExecute,   // PreEquation
  Otherwise,    // Equation, "owise"
  Variant,      // Equation
  Print,        // StatementAttributeInfo--not a `PreEquation`
  Narrowing,    // Rule
  Bad,          // A malformed pre-equation
}
pub type PreEquationAttributes = BitFlags<PreEquationAttribute>;


pub struct PreEquation {
  pub name      : Option<IString>,
  pub attributes: PreEquationAttributes,
  pub conditions: Conditions,

  pub lhs_term  : BxTerm,
  pub kind      : PreEquationKind,
}


/// Representation of Rule, Equation, Sort Constraint/Membership Axiom.
pub enum PreEquationKind {
  Equation {
    rhs_term: BxTerm,
  },

  Rule {
    rhs_term: BxTerm,
  },

  // Membership Axiom ("Sort constraint")
  Membership {
    sort: RcSort,
  },

  // StrategyDefinition
}
