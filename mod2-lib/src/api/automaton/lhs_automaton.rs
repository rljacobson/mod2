/*!

The automaton that matches the LHS.

*/

use mod2_abs::Outcome;
use crate::{
  api::{
    dag_node::DagNodePtr,
    subproblem::MaybeSubproblem,
  },
  core::{
    sort::SortPtr,
    substitution::Substitution,
  },
};
use crate::core::VariableIndex;

pub type BxLHSAutomaton = Box<dyn LHSAutomaton>;

pub trait LHSAutomaton {
  fn match_(
    &mut self,
    subject : DagNodePtr,
    solution: &mut Substitution,
    // returned_subproblem: Option<&mut dyn Subproblem>,
    // extension_info: Option<&mut dyn ExtensionInfo>,
  ) -> (bool, MaybeSubproblem);


  // In Maude this is a method on DagNode.
  fn match_variable(
    &self,
    mut dag_node             : DagNodePtr,
    index                    : VariableIndex,
    sort                     : SortPtr,
    copy_to_avoid_overwriting: bool,
    solution                 : &mut Substitution,
    // extension_info: Option<&ExtensionInfo>
  ) -> (bool, MaybeSubproblem) {
    // if let Some(ext_info) = extension_info {
    //   return self.match_variable_with_extension(index, sort, solution, returned_subproblem, ext_info);
    // }
    let maybe_dag_node = solution.get(index);
    match maybe_dag_node {
      None => {
        if let (Outcome::Success, maybe_subproblem) = dag_node.check_sort(sort) {
          let dag_node_ref = if copy_to_avoid_overwriting {
            dag_node.make_clone()
          } else {
            dag_node
          };
          solution.bind(index, Some(dag_node_ref));
          (true, maybe_subproblem)
        } else {
          (false, None)
        }
      }
      Some(existing_d) => {
        if dag_node.compare(existing_d).is_eq() {
          (true, None)
        } else {
          (false, None)
        }
      }
    }
  }
}
