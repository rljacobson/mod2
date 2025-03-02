/*!

Equations, rules, membership axioms, and strategies can have optional
conditions that must be satisfied in order for the pre-equation to
apply. Conditions are like a "lite" version of `PreEquation`.

*/

use std::fmt::Display;
use crate::api::term::BxTerm;
use crate::core::sort::sort_spec::BxSortSpec;

pub type Conditions  = Vec<BxCondition>;
pub type BxCondition = Box<Condition>;

pub enum Condition {
  /// Equality conditions, `x = y`.
  ///
  /// Boolean expressions are shortcut versions of equality conditions of the form `expr = true`.
  Equality {
    lhs_term: BxTerm,
    rhs_term: BxTerm
  },

  /// Also called a sort test condition, `X :: Y`
  SortMembership {
    lhs_term: BxTerm,
    sort    : BxSortSpec
  },

  /// Also called an assignment condition, `x := y`
  Match {
    lhs_term: BxTerm,
    rhs_term: BxTerm
  },

  /// Also called a rule condition, `x => y`
  Rewrite {
    lhs_term: BxTerm,
    rhs_term: BxTerm
  },
}

impl Display for Condition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {

      Condition::Equality { lhs_term, rhs_term } => {
        write!(f, "{} = {}", *lhs_term, *rhs_term)
      }

      Condition::SortMembership { lhs_term, sort } => {
        write!(f, "{} :: {}", *lhs_term, *sort)
      }

      Condition::Match { lhs_term, rhs_term } => {
        write!(f, "{} := {}", *lhs_term, *rhs_term)
      }

      Condition::Rewrite { lhs_term, rhs_term } => {
        write!(f, "{} => {}", *lhs_term, *rhs_term)
      }

    }
  }
}
