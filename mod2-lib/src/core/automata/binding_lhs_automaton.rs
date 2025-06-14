/*!

Struct for a left-hand-side automata that just binds a variable and call another lhs automaton to do the
real work.

*/

use crate::{
  api::{
    automaton::{BxLHSAutomaton, LHSAutomaton},
    dag_node::DagNodePtr,
    subproblem::MaybeSubproblem
  },
  core::substitution::Substitution,
};
use crate::core::VariableIndex;

pub(crate) struct BindingLHSAutomaton {
  variable_index:    VariableIndex,
  real_lhs_automata: BxLHSAutomaton,
}

impl BindingLHSAutomaton {
  pub fn new(variable_index: VariableIndex, real_lhs_automata: BxLHSAutomaton) -> BxLHSAutomaton {
    Box::new(
    BindingLHSAutomaton {
      variable_index,
      real_lhs_automata,
    }
    )
  }
}


impl LHSAutomaton for BindingLHSAutomaton {
  fn match_(
    &mut self,
    subject: DagNodePtr,
    solution: &mut Substitution,
    // extension_info: Option<&mut dyn ExtensionInfo>,
  ) -> (bool, MaybeSubproblem) {
    let (matched, maybe_subproblem) = self.real_lhs_automata.match_(subject, solution);
    if matched {
      solution.bind(self.variable_index, Some(subject));
      return (matched, maybe_subproblem);
    }

    (false, None)
  }
}
