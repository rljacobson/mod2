use crate::api::automaton::LHSAutomaton;
use crate::api::dag_node::DagNodePtr;
use crate::api::subproblem::MaybeSubproblem;
use crate::api::term::TermPtr;
use crate::core::substitution::Substitution;

pub struct NonalgebraicLHSAutomaton {
  term: TermPtr
}

impl NonalgebraicLHSAutomaton {
  pub fn new(term: TermPtr) -> NonalgebraicLHSAutomaton {
    NonalgebraicLHSAutomaton { term }
  }
}

impl LHSAutomaton for NonalgebraicLHSAutomaton {
  fn match_(&mut self, subject: DagNodePtr, _solution: &mut Substitution) -> (bool, MaybeSubproblem) {
    (self.term.compare_dag_node(subject).is_eq(), None)
  }
}