/*!

Equations, rules, membership axioms, and strategies can have optional
conditions that must be satisfied in order for the pre-equation to
apply. Conditions are like a "lite" version of `PreEquation`.

*/

use crate::theory::term::BxTerm;
use crate::core::sort::sort_spec::BxSortSpec;

pub type Conditions  = Vec<BxCondition>;
pub type BxCondition = Box<Condition>;

pub enum Condition {
  /// Boolean expressions are shortcut versions of equality conditions `expr = true`.
  Equality {
    lhs_term: BxTerm,
    rhs_term: BxTerm
  },

  /// Also called a sort test condition
  SortMembership {
    lhs_term: BxTerm,
    sort    : BxSortSpec
  },

  /// Also called an assignment condition
  Match {
    lhs_term: BxTerm,
    rhs_term: BxTerm
  },

  /// Also called a rule  condition
  Rewrite {
    lhs_term: BxTerm,
    rhs_term: BxTerm
  },
}
