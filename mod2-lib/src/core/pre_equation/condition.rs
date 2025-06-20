/*!

Equations, rules, membership axioms, and strategies can have optional
conditions that must be satisfied in order for the pre-equation to
apply. Conditions are like a "lite" version of `PreEquation`.

*/

use std::fmt::Display;
use std::ops::Deref;
use mod2_abs::NatSet;
use crate::{
  api::{
    automaton::BxLHSAutomaton,
    automaton::LHSAutomaton,
    subproblem::Subproblem,
    term::{
      BxTerm,
      TermPtr
    }
  },
  core::{
    automata::RHSBuilder,
    rewriting_context::RewritingContext,
    sort::SortPtr,
    substitution::Substitution,
    StateTransitionGraph,
    VariableInfo,
    VariableIndex
  }
};
use Condition::*;


pub type Conditions  = Vec<BxCondition>;
pub type BxCondition = Box<Condition>;

/// Holds state information used in solving condition fragments.
pub enum ConditionState {
  Assignment {
    saved:       Substitution,
    rhs_context: Box<RewritingContext>,
    subproblem:  Box<dyn Subproblem>,
    succeeded:   bool,
  },

  Rewrite {
    state_graph: StateTransitionGraph,
    matcher:     Box<dyn LHSAutomaton>,
    saved:       Substitution,
    subproblem:  Box<dyn Subproblem>,
    explore:     i32,
    edge_count:  u32,
  },
}

pub enum Condition {
  /// Equality conditions, `x = y`.
  ///
  /// Boolean expressions are shortcut versions of equality conditions of the form `expr = true`.
  Equality {
    lhs_term : BxTerm,
    rhs_term : BxTerm,
    builder  : RHSBuilder,
    lhs_index: VariableIndex,
    rhs_index: VariableIndex,
  },

  /// Also called a sort test condition, `X :: Y`
  SortMembership {
    lhs_term : BxTerm,
    sort     : SortPtr,
    builder  : RHSBuilder,
    lhs_index: VariableIndex,
  },

  /// Also called an Assignment condition, `x := y`
  Match {
    lhs_term   : BxTerm,
    rhs_term   : BxTerm,
    builder    : RHSBuilder,
    lhs_matcher: Option<BxLHSAutomaton>,
    rhs_index  : VariableIndex,
  },

  /// Also called a rule condition, `x => y`
  Rewrite {
    lhs_term   : BxTerm,
    rhs_term   : BxTerm,
    builder    : RHSBuilder,
    rhs_matcher: Option<BxLHSAutomaton>,
    lhs_index  : VariableIndex,
  },
}


impl Condition {
  fn lhs_term(&self) -> TermPtr {
    match self {
      Equality { lhs_term, .. } 
      | SortMembership { lhs_term, .. }
      | Match { lhs_term, .. }
      | Rewrite { lhs_term, .. } => {
        lhs_term.as_ptr()
      }
    }
  }
  
  pub fn check(&mut self, variable_info: &mut VariableInfo, bound_variables: &mut NatSet) {
    let mut unbound_variables = NatSet::new();

    // Handle variables in the pattern.
    match self {
      Equality { lhs_term, .. } | SortMembership { lhs_term, .. } | Rewrite { lhs_term, .. } => {
        lhs_term.normalize(true);
        lhs_term.index_variables(variable_info);
        variable_info.add_condition_variables(lhs_term.occurs_below());
        unbound_variables.union_in_place(lhs_term.occurs_below());
      }
      Match { lhs_term, .. } => {
        lhs_term.normalize(true);
        lhs_term.index_variables(variable_info);
        variable_info.add_condition_variables(lhs_term.occurs_below());
      }
    }

    assert!(
      !bound_variables.is_superset(self.lhs_term().occurs_below()),
      "{}: all the variables in the left-hand side of Match condition fragment {} are bound before the
    matching takes place.",   self.lhs_term(),
      self
    );

    // Handle variables in the subject.
    match self {
      Equality { rhs_term, .. } | Match { rhs_term, .. } | Rewrite { rhs_term, .. } => {
        rhs_term.normalize(true);
        rhs_term.index_variables(variable_info);
        variable_info.add_condition_variables(rhs_term.occurs_below());

        // Check for variables that are used before they are bound.
        unbound_variables.union_in_place(rhs_term.occurs_below());
      }
      _ => { /* noop */ }
    }

    unbound_variables.difference_in_place(bound_variables);
    variable_info.add_unbound_variables(&unbound_variables);

    // We will bind these variables.
    match &self {
      Rewrite { lhs_term, .. } | Match { lhs_term, .. } => {
        bound_variables.union_in_place(lhs_term.occurs_below());
      }
      _ => { /* noop */ }
    }
  }

  pub fn solve(&mut self, find_first: bool, solution: &mut RewritingContext, _state: &mut Vec<ConditionState>) -> bool {
    match self {
      Match { .. } => todo!("Implement match condition solve"),

      Equality {
        builder,
        lhs_index,
        rhs_index,
        ..
      } => {
        if !find_first {
          return false;
        }

        builder.safe_construct(&mut solution.substitution);
        let lhs_root = solution.substitution.get(*lhs_index);
        let mut lhs_context = RewritingContext::new(lhs_root);
        let rhs_root = solution.substitution.get(*rhs_index);
        let mut rhs_context = RewritingContext::new(rhs_root);

        lhs_context.reduce();
        solution.add_counts_from(&lhs_context);
        rhs_context.reduce();
        solution.add_counts_from(&rhs_context);

        (*lhs_context.root.unwrap().as_ref()).deref() == (*rhs_context.root.unwrap().as_ref()).deref()
      }

      Rewrite { .. } => todo!("Implement rewrite condition solve"),

      SortMembership { .. } => todo!("Implement sort test condition solve"),
    }
  }
}

impl Display for Condition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {

      Condition::Equality { lhs_term, rhs_term, .. } => {
        write!(f, "{} = {}", *lhs_term, *rhs_term)
      }

      Condition::SortMembership { lhs_term, sort, .. } => {
        write!(f, "{} : {}", *lhs_term, sort)
      }

      Condition::Match { lhs_term, rhs_term, .. } => {
        write!(f, "{} := {}", *lhs_term, *rhs_term)
      }

      Condition::Rewrite { lhs_term, rhs_term, .. } => {
        write!(f, "{} => {}", *lhs_term, *rhs_term)
      }

    }
  }
}
