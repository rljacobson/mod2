/*!

A `PreEquation` is just a superclass for equations, rules, sort constraints, and strategies (the last of which is not
implemented.) The subclass is implemented as enum `PreEquationKind`.

*/

pub mod condition;

use std::fmt::{Display, Formatter};
use enumflags2::{bitflags, BitFlags};

use mod2_abs::{join_string, IString};

use crate::{
  core::{
    pre_equation::condition::Conditions,
    sort::{
      sort::SortPtr,
      sort_spec::BxSortSpec
    }
  },
  theory::term::BxTerm,
};


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

impl Display for PreEquationAttribute {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      PreEquationAttribute::Compiled   => write!(f, "compiled"),
      PreEquationAttribute::NonExecute => write!(f, "nonexecute"),
      PreEquationAttribute::Otherwise  => write!(f, "otherwise"),
      PreEquationAttribute::Variant    => write!(f, "variant"),
      PreEquationAttribute::Print      => write!(f, "print"),
      PreEquationAttribute::Narrowing  => write!(f, "narrowing"),
      PreEquationAttribute::Bad        => write!(f, "bad"),
    }
  }
}

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
    sort_spec: BxSortSpec,
  },

  // StrategyDefinition
}

impl Display for PreEquation {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self.kind {

      PreEquationKind::Equation { rhs_term } => {
        write!(f, "equation {} = {}", self.lhs_term,  rhs_term)?;
      }

      PreEquationKind::Rule { rhs_term } => {
        write!(f, "rule {} => {}", self.lhs_term,  rhs_term)?;

      }

      PreEquationKind::Membership { sort_spec } => {
        write!(f, "membership {} :: {}", self.lhs_term,  sort_spec)?;

      }

    }

    // conditions
    if !self.conditions.is_empty() {
      write!(
        f,
        " if {}",
        join_string(self.conditions.iter(), " ⋀ ")
      )?;
    }

    // attributes
    if !self.attributes.is_empty() {
      write!(
        f,
        " [{}]",
        join_string(self.attributes.iter(), ", ")
      )?;
    }

    write!(f, ";")
  }
}
