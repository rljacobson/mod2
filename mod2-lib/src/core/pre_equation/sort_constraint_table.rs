/*!

Membership constraints, also called sort constraints, specify subsort relationships.

The `SortConstraintTable` class manages a collection of sort constraints indexed under a
symbol for efficient sort constraint matching and application during sort computation.

### Core Functionality

**Sort Constraint Management**: The class maintains a private vector of
*`SortConstraint*` objects and provides methods to add constraints via
*`offer_sort_constraint()`, which calls the pure virtual `accept_sort_constraint()`
*method that subclasses must implement to filter which constraints they accept.

**Sort Constraint Application**: The primary functionality is provided by:
- `constrain_to_smaller_sort()`: Public interface that delegates to the private implementation if constraints exist
- `constrain_to_smaller_sort_2()`: The core implementation that attempts to apply sort constraints to refine a DAG
  node's sort to a smaller (more specific) sort

### Sort Constraint Processing Strategy

The constraint application uses an iterative refinement approach:
1. **Ordering**: Sort constraints are ordered with smallest sorts first using `sort_constraint_lt()`
2. **Multi-pass filtering**: `order_sort_constraints()` uses successive passes to "comb out" usable constraints
   as the symbol's sort is refined
3. **Retry mechanism**: When a constraint successfully applies, the process restarts from the beginning since
   earlier constraints may now be applicable

### Limited Context Handling

The implementation includes special handling for limited rewriting contexts, which don't support sort constraint
application and are used for functionality that doesn't require sort constraints.

### Additional Methods

- `compile_sort_constraints()`: Compiles all stored sort constraints for execution
- `sort_constraint_free()`: Checks if the table contains no sort constraints
- `constrain_to_exact_sort()`: Currently delegates to the same implementation as `constrain_to_smaller_sort()`

### Usage Pattern

Subclasses inherit from `SortConstraintTable` and implement `accept_sort_constraint()`
to define which sort constraints they handle, then use the public constraint
application methods during sort computation to refine DAG node sorts.

## Notes

The class is part of Maude's core sort system and integrates with the tracing
system for debugging sort constraint applications. The iterative refinement
approach handles the complex interdependencies between sort constraints
where applying one constraint may enable others to become applicable.

*/

use std::cmp::Ordering;
use mod2_abs::warning;
use crate::{
  core::{
    sort::{sort_leq_index, index_leq_sort},
    rewriting_context::RewritingContext,
  },
  api::DagNodePtr,
};
use super::{
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

  /// Attempts to constrain the given DAG node to a smaller sort by applying applicable
  /// sort constraints. Iterates through sort constraints in order from smallest to largest
  /// sort, applying the first matching constraint that can reduce the node's sort. If a
  /// constraint successfully applies, the process restarts from the beginning to check
  /// if earlier constraints can now apply to the newly constrained sort. Does nothing
  /// if the symbol has no sort constraints or if the rewriting context is limited.
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
                  .match_(subject.clone(), &mut context.substitution, None)
              {
                if subproblem.is_none()
                    || subproblem.as_mut().map_or(false, |s| s.solve(true, context))
                {

                  let condition = !sort_constraint.has_condition()
                    || match subproblem {
                    None => sort_constraint.check_condition(subject.clone(), context, None),
                    Some(mut sp) => sort_constraint.check_condition(subject.clone(), context, Some(sp.as_mut()))
                  };

                  if condition {
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
