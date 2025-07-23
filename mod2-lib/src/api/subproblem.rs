/*!

`Subproblem`s are designed to be stateful iterators that can generate multiple
solutions through successive calls to `Subproblem::solve()`. The `find_first` parameter
controls whether you're starting fresh or continuing from where you left off.

A `Subproblem` corresponds roughly to the `MatchGenerator`s of Loris. These traits must be
derived from for equational theories that need to generate matching or unification subproblems.

This module defines the following implementors:

- `VariableAbstractionSubproblem`
- `SubproblemSequence`
-

# Eker96

## Matching Phase

Here the result of a match is a partial substitution which contains variables that can easily be
determined to have the same value in all matching substitutions; together with a subproblem object
which is a compact representation of the possible values for the variables not mentioned in the
partial substitution. For a simple pattern the partial substitution might contain bindings for all
the variables in the pattern in which case the empty subproblem object denoted by $\emptyset$ is returned.
Of course the matching phase could fail altogether in which case the pair (fail, $\emptyset$) is returned.

## Subproblem Solving Phase

For many simple patterns this phase will be unnecessary as the matching phase will have uniquely
bound all the variables. For more complex patterns we are left with a partial substitution
and a subproblem object which may contain nested subproblem sub-objects. In the subproblem
solving phase the subproblem object is searched for consistent sets of solutions to the unbound
variables; each such set corresponds to a different solution to the original matching problem.

For implementation purposes subproblem objects actually contain state information to record which
possibilities have already been tried and the returned subproblem object is really the original subproblem
object with its state updated. Thus, solutions can be extracted from the subproblem object as needed.

*/
use mod2_abs::Outcome;

use crate::{
  api::DagNodePtr,
  core::{
    LocalBindings,
    SortIndex,
    VariableIndex,
    rewriting_context::RewritingContext,
    sort::SortPtr,
    substitution::Substitution,
  }
};
use super::automaton::LHSAutomaton;

pub type MaybeSubproblem        = Option<Box<dyn Subproblem>>;
pub type MaybeSubproblemRef<'s> = Option<&'s mut dyn Subproblem>;

/// Represents a subproblem of a matching problem.
pub trait Subproblem {
  fn solve(&mut self, find_first: bool, context: &mut RewritingContext) -> bool;
}

pub struct VariableAbstractionSubproblem {
  pub abstracted_pattern  : Box<dyn LHSAutomaton>,
  pub abstraction_variable: VariableIndex,
  pub variable_count      : u32,
  pub difference          : Option<LocalBindings>,
  pub subproblem          : MaybeSubproblem,
  pub local               : Substitution, // Todo: How does this differ from `difference`?
  pub solved              : bool,
}

impl VariableAbstractionSubproblem {
  pub fn new(
    abstracted_pattern  : Box<dyn LHSAutomaton>,
    abstraction_variable: VariableIndex,
    variable_count      : u32
  ) -> Self
  {
    VariableAbstractionSubproblem {
      abstracted_pattern,
      abstraction_variable,
      variable_count,
      difference: Some(LocalBindings::default()),
      subproblem: None,
      local     : Default::default(),
      solved    : false,
    }
  }
}

impl Subproblem for VariableAbstractionSubproblem {
  fn solve(&mut self, find_first: bool, context: &mut RewritingContext) -> bool {
    if find_first {
      self.local.copy_from_substitution(&context.substitution);

      let v = context.substitution.get(self.abstraction_variable);
      assert!(v.is_some(), "Unbound abstraction variable");
      let v = v.unwrap();

      // Todo: What about the potential subproblem? Is it pushed to self.subproblem? If so, why return it?
      if let (false, _) = self.abstracted_pattern.match_(v, &mut self.local, None) {
        return false;
      }

      self.difference = self.local.subtract(&context.substitution);
      if let Some(difference) = self.difference.as_mut() {
        difference.assert(&mut context.substitution);
      }

      if let Some(subproblem) = &mut self.subproblem {
        if subproblem.solve(true, context) {
          return true;
        }
      } else {
        return true;
      }
    } else {
      if let Some(subproblem) = &mut self.subproblem {
        if subproblem.solve(false, context) {
          return true;
        }
      }
    }

    if let Some(difference) = self.difference.as_mut() {
      difference.retract(&mut context.substitution);
      self.difference = None;
    }

    self.subproblem = None;
    false
  }
}

/// Maude calls this SubproblemAccumulator
pub struct SubproblemSequence {
  sequence: Vec<Box<dyn Subproblem>>,
}

impl SubproblemSequence {
  pub fn new() -> Self {
    SubproblemSequence { sequence: vec![] }
  }

  pub fn add(&mut self, subproblem: Box<dyn Subproblem>) {
    self.sequence.push(subproblem);
  }

  pub fn extract_subproblem(mut self) -> Option<Box<dyn Subproblem>> {
    if self.sequence.is_empty() {
      None
    } else if self.sequence.len() == 1 {
      Some(self.sequence.pop().unwrap())
    } else {
      Some(Box::new(self))
    }
  }

  pub fn push(&mut self, subproblem: Box<dyn Subproblem>) {
    self.sequence.push(subproblem);
  }
}

impl Subproblem for SubproblemSequence {
  fn solve(&mut self, mut find_first: bool, context: &mut RewritingContext) -> bool {
    let len = self.sequence.len();
    let mut i: isize = match find_first {
      true => 0isize,
      false => len as isize - 1,
    };

    loop {
      find_first = self.sequence[i as usize].solve(find_first, context);
      if find_first {
        i += 1;
        if i == len as isize {
          break;
        }
      } else {
        i -= 1;
        if i < 0 {
          break;
        }
      }
    }

    find_first
  }
}


pub struct SortCheckSubproblem {
  pub subject: DagNodePtr,
  pub sort   : SortPtr,
  pub result : Outcome
}

impl SortCheckSubproblem {
  pub fn new(subject: DagNodePtr, sort: SortPtr) -> SortCheckSubproblem {
    SortCheckSubproblem{
      subject,
      sort,
      result: Outcome::Undecided
    }
  }
}

impl Subproblem for SortCheckSubproblem {
  fn solve(&mut self, find_first: bool, solution: &mut RewritingContext) -> bool {
    if !find_first {
      return false; // Maude: Only ever one way to solve; otherwise infinite loop.
    }

    if self.result == Outcome::Undecided {
      self.result = self.subject.check_sort_in_context(self.sort, solution);
    }

    self.result == Outcome::Success
  }
}
