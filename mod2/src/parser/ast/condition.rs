/*!

Represents a condition attached to a rule, equation, or membership axiom.

The terminology in Maude is a little confused.

```ebnf
ConditionSpec := "if" Condition (AndOp Condition)* ;

Condition :=
    EqualityCondition
    | MatchCondition
    | MembershipCondition
    | RewriteCondition
    ;

EqualityCondition :=
    Term EqualOp Term # Regular Equation
    | Term            # Boolean Expression (short for term = true)
    ;

MatchCondition := Term MatchOp Term ;

MembershipCondition := Term SortOp SortSpec ;

RewriteCondition := Term RuleOp Term ;
```
*/
use mod2_abs::{
  HashMap,
  IString
};

use mod2_lib::{
  core::{
    pre_equation::condition::Condition,
    sort::collection::SortCollection
  },
  api::{
    symbol_core::SymbolPtr,
    term::Term,
  },
};
use mod2_lib::api::built_in::get_built_in_symbol;
use mod2_lib::api::free_theory::FreeTerm;
use crate::{
  parser::ast::{
    BxFunctorSortAST,
    BxTermAST
  },
};

pub(crate) enum ConditionAST {
  /// Represents equality conditions of the form `lhs = rhs`.
  /// Equality conditions include any BOOL-valued condition as a special case (including inequality comparisons),
  /// though they are captured in the `ConditionAST::Boolean` variant.
  Equality       { lhs: BxTermAST, rhs : BxTermAST },
  /// Also called membership constraint or sort test conditions
  SortMembership { lhs: BxTermAST, sort: BxFunctorSortAST },
  /// Also called an assignment condition
  Match          { lhs: BxTermAST, rhs : BxTermAST },
  /// Also called a rule condition
  Rewrite        { lhs: BxTermAST, rhs : BxTermAST },
  /// Boolean expressions are shortcut versions of equality conditions `expr = true`.
  Boolean        (BxTermAST)
}

impl ConditionAST {
  pub fn construct(
    &self,
    symbols: &mut HashMap<IString, SymbolPtr>,
    sorts  : &mut SortCollection
  ) -> Condition
  {
    match self {

      ConditionAST::Equality { lhs, rhs } => {
        Condition::Equality {
          lhs_term: Box::new(lhs.construct(symbols)),
          rhs_term: Box::new(rhs.construct(symbols)),
        }
      }

      ConditionAST::SortMembership { lhs, sort } => {
        let sort = sort.construct(sorts);
        Condition::SortMembership {
          lhs_term: Box::new(lhs.construct(symbols)),
          sort
        }
      }

      ConditionAST::Match { lhs, rhs } => {
        Condition::Match {
          lhs_term: Box::new(lhs.construct(symbols)),
          rhs_term: Box::new(rhs.construct(symbols)),
        }
      }

      ConditionAST::Rewrite { lhs, rhs } => {
        Condition::Rewrite {
          lhs_term: Box::new(lhs.construct(symbols)),
          rhs_term: Box::new(rhs.construct(symbols)),
        }
      }

      ConditionAST::Boolean(lhs) => {
        // The RHS is just boolean true.
        let true_symbol = get_built_in_symbol("true").unwrap();
        Condition::Equality {
          lhs_term: Box::new(lhs.construct(symbols)),
          rhs_term: Box::new(FreeTerm::new(true_symbol)),
        }
      }

    }

  }
}
