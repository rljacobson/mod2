/*!

Struct for a left-hand-side automata that just binds a variable and call another lhs automaton to do the
real work.

*/

use crate::{
  api::{
    subproblem::MaybeSubproblem,
    dag_node::DagNodePtr,
    automaton::{BxLHSAutomaton, LHSAutomaton},
    variable_theory::VariableIndex
  },
  core::substitution::Substitution,
};

pub(crate) struct BindingLHSAutomaton {
  variable_index:    VariableIndex,
  real_lhs_automata: BxLHSAutomaton,
}

impl BindingLHSAutomaton {
  pub fn new(variable_index: VariableIndex, real_lhs_automata: BxLHSAutomaton) -> Self {
    BindingLHSAutomaton {
      variable_index,
      real_lhs_automata,
    }
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
