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
    sort::SortCollection
  },
  api::{
    built_in::{get_built_in_symbol, BoolTerm},
    free_theory::FreeTerm,
    BxTerm,
    SymbolPtr,
    Term, 
  },
};

use crate::parser::ast::{
  BxFunctorSortAST,
  BxTermAST,
  BxSortIdAST,
};

pub(crate) enum ConditionAST {
  /// Represents equality conditions of the form `lhs = rhs`.
  /// Equality conditions include any BOOL-valued condition as a special case (including inequality comparisons),
  /// though they are captured in the `ConditionAST::Boolean` variant.
  Equality       { lhs: BxTermAST, rhs : BxTermAST },
  /// Also called membership constraint or sort test conditions
  SortMembership { lhs: BxTermAST, sort: BxSortIdAST },
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
    symbols  : &mut HashMap<IString, SymbolPtr>,
    sorts    : &mut SortCollection,
    variables: &mut HashMap<IString, BxTerm>
  ) -> Condition
  {
    match self {

      ConditionAST::Equality { lhs, rhs } => {
        Condition::Equality {
          lhs_term: lhs.construct(symbols, sorts, variables),
          rhs_term: rhs.construct(symbols, sorts, variables),
          builder: Default::default(),
          lhs_index: 0,
          rhs_index: 0,
        }
      }

      ConditionAST::SortMembership { lhs, sort } => {
        let sort = sort.construct(sorts);
        Condition::SortMembership {
          lhs_term: lhs.construct(symbols, sorts, variables),
          sort,
          builder: Default::default(),
          lhs_index: 0,
        }
      }

      ConditionAST::Match { lhs, rhs } => {
        Condition::Match {
          lhs_term: lhs.construct(symbols, sorts, variables),
          rhs_term: rhs.construct(symbols, sorts, variables),
          builder: Default::default(),
          lhs_matcher: None,
          rhs_index: 0,
        }
      }

      ConditionAST::Rewrite { lhs, rhs } => {
        Condition::Rewrite {
          lhs_term: lhs.construct(symbols, sorts, variables),
          rhs_term: rhs.construct(symbols, sorts, variables),
          builder: Default::default(),
          rhs_matcher: None,
          lhs_index: 0,
        }
      }

      ConditionAST::Boolean(lhs) => {
        // The RHS is just boolean true.
        Condition::Equality {
          lhs_term: lhs.construct(symbols, sorts, variables),
          rhs_term: Box::new(BoolTerm::new(true)),
          builder: Default::default(),
          lhs_index: 0,
          rhs_index: 0,
        }
      }

    }

  }
}
