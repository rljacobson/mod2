/*!

Membership constraints, also called sort constraints, specify subsort relationships. 

*/

use std::{
  cmp::Ordering,
  ops::{Deref, DerefMut}
};
use mod2_abs::warning;
use crate::{
  core::{
    sort::{sort_leq_index, index_leq_sort},
    rewriting_context::RewritingContext,
  },
  api::dag_node::DagNodePtr,
};
use super::{
  PreEquation as SortConstraint,
  PreEquationKind,
  PreEquationPtr as SortConstraintPtr
};



#[derive(Default)]
pub struct SortConstraintTable {
  constraints: Vec<SortConstraintPtr>,
  complete   : bool,
}

impl SortConstraintTable {
  pub fn new() -> SortConstraintTable {
    SortConstraintTable::default()
  }
  
  
  #[inline(always)]
  pub fn offer_sort_constraint(&mut self, sort_constraint: SortConstraintPtr) {
    if self.accept_sort_constraint(sort_constraint) {
      self.constraints.push(sort_constraint);
    }
  }

  #[inline(always)]
  pub fn sort_constraint_free(&self) -> bool {
    self.constraints.is_empty()
  }

  #[inline(always)]
  pub fn safe_to_inspect_sort_constraints(&self) -> bool {
    self.complete
  }

  /// Sort constraints are sorted in the order: largest index (smallest sort) first
  fn sort_constraint_lt(lhs: SortConstraintPtr, rhs: SortConstraintPtr) -> Ordering {
    if let PreEquationKind::Membership { sort: lhs_sort, .. } = &lhs.pe_kind {
      if let PreEquationKind::Membership { sort: rhs_sort, .. } = &rhs.pe_kind {
        // reverse order: large index --> small sort
        return rhs_sort.index_within_kind.cmp(&lhs_sort.index_within_kind);
        // IDEA: might want to weaken comparison and do a stable_sort()
        // BUT: stable_sort() requires a strict weak ordering - much stronger than
        // the partial ordering we have on sorts.
      }
    }
    unreachable!("Non SortConstraint PreEquation used in SortConstraint context. This is a bug.")
  }

  fn order_sort_constraints(&mut self) {
    // `self.constraints` may contain sort constraints with variable lhs which have
    // too low a sort to ever match our symbol. However the sort of our symbol
    // is itself affected by sort constraints. So we "comb" out usable sort
    // constraints in successive passes; this is inefficient but we expect the number
    // of sort constraints to be very small so it's not worth doing anything smarter.
    self.complete = true; // Not really complete until we've finished, but pretend it is.
    let sort_constraint_count = self.constraints.len();
    if sort_constraint_count == 0 {
      return;
    }
    let mut all: Vec<Option<SortConstraintPtr>> = Vec::with_capacity(sort_constraint_count);
    all.extend(self.constraints.drain(..).map(Some));
    let mut added_sort_constraint: bool;
    
    // Repeatedly loop over the `all` vector until we no longer add a sort constraint.
    loop {
      added_sort_constraint = false;
      for i in 0..sort_constraint_count {
        if let Some(sc) = &all[i] {
          // Because we set table_complete = true; accept_sort_constraint() may
          // inspect the table of sort_constraints accepted so far and make
          // a finer distinction than it could in offer_sort_constraint().
          if self.accept_sort_constraint(*sc) {
            self.constraints.push(*sc);
            all[i] = None;
            added_sort_constraint = true;
          }
        }
      }
      if !added_sort_constraint {
        break;
      }
    }
    self.constraints
        .sort_by(|a, b| { Self::sort_constraint_lt(*a, *b) });
  }

  /*
  #[inline(always)]
  fn compile_sort_constraints(&mut self) {
    for constraint in self.constraints {
      constraint.compile(true);
    }
  }
  */

  // Placeholder for the actual implementations of these methods
  fn accept_sort_constraint(&self, _sort_constraint: SortConstraintPtr) -> bool {
    unimplemented!()
  }

  pub(crate) fn constrain_to_smaller_sort(&mut self, mut subject: DagNodePtr, context: &mut RewritingContext) {
    if self.sort_constraint_free() {
      return;
    }

    if context.is_limited() {
      // Limited rewriting contexts don't support sort constraint application and
      // are only used for functionality that doesn't support sort constraints.
      warning!(1, "ignoring sort constraints for {} because context is limited", subject);
      return;
    }

    let mut current_sort_index = subject.sort_index();

    // We try sort constraints, smallest sort first until one applies or
    // all remaining sort constraints have sort >= than our current sort.
    // Whenever we succeed in applying a sort constraint we start again
    // with the new sort, because earlier sort constraints (via collapse
    // or variable lhs patterns) may be able to test this new sort.
    'retry: loop {
      for sort_constraint in self.constraints.iter_mut() {
        if let PreEquationKind::Membership { sort, .. } = sort_constraint.pe_kind {
          if index_leq_sort(current_sort_index, &sort) {
            // Done!
            return;
          }

          if sort_leq_index(&sort, current_sort_index) {
            {
              // not equal because of previous test
              let variable_count = sort_constraint.variable_info.protected_variable_count();
              context.substitution.clear_first_n(variable_count as usize);
            }
            if let Some(lhs_automaton) = sort_constraint.lhs_automaton.as_mut()
            {
              if let (true, mut subproblem) = lhs_automaton
                  .as_mut()
                  .match_(subject.clone(), &mut context.substitution)
              {
                if subproblem.is_none() || subproblem.as_mut().unwrap().solve(true, context) {
                  
                  if !sort_constraint.has_condition()
                      || sort_constraint.check_condition(subject.clone(), context, subproblem)
                  {
                    // subproblem.take(); // equivalent to delete sp in C++
                    /* ToDo: Implement tracing
                    if trace_status() {
                      context.trace_pre_eq_application(
                        Some(subject.clone()),
                        Some(&*sort_constraint),
                        RewriteType::Normal,
                      );
                      if context.trace_abort() {
                        context.finished();
                        return;
                      }
                    }
                    */
                    context.membership_count += 1;
                    context.finished();

                    current_sort_index = sort.index_within_kind;
                    subject.set_sort_index(current_sort_index);
                    continue 'retry;
                  }
                }
              }
            }

          }
          context.finished();
        } else {
          unreachable!("Found a non SortConstraint. This is a bug.");
        }
      }

      break;
    }
  }
}