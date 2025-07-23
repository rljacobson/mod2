/*!

In Maude, RewriteSearchState is a subclass of SearchState.

*/

use mod2_abs::{NatSet, UnsafePtr};
use crate::{
  api::{
    Subproblem,
    BxTerm
  },
  core::{
    gc::root_container::{BxRootVec, RootVec},
    pre_equation::{
      PreEquationPtr,
      condition::ConditionState
    },
    state_transition_graph::{
      PositionState,
      PositionDepth,
      PositionStateDepthSentinel,
      StateFlag,
      StateFlags
    },
    rewriting_context::BxRewritingContext,
    RuleIndex,
  }
};
use crate::api::LHSAutomaton;
use crate::core::{VariableIndex, VariableInfo};

pub struct RewriteSearchState {
  position_state: PositionState,
  context       : BxRewritingContext,
  pre_equation  : Option<PreEquationPtr>,

  // Initial (partial) substitution
  substitution_variables: Vec<BxTerm>,
  substitution_values   : BxRootVec,

  // For backtracking over matches
  matching_subproblem: Option<Box<dyn Subproblem>>,

  // For backtracking of solutions to a rule condition
  trial_ref      : RuleIndex,
  condition_stack: Vec<ConditionState>,
  rule_index     : RuleIndex,
}

impl RewriteSearchState {
  pub fn new(
    context  : BxRewritingContext,
    mut flags: StateFlags,
    min_depth: PositionDepth,
    max_depth: PositionDepth,
  ) -> Self {
    flags.insert(StateFlag::RespectFrozen);

    /*
    if label != Label::NONE {
      assert!(
        !flags.contains(StateFlags::SET_UNREWRITABLE),
        "shouldn't set unrewritable flag if only looking at rules with a given label"
      );
      assert!(
        !flags.contains(StateFlags::SET_UNSTACKABLE),
        "shouldn't set unstackable flag if only looking at rules with a given label"
      );
    }
    */
    if flags.contains(StateFlag::AllowNonexec) {
      assert!(
        !flags.contains(StateFlag::RespectUnrewritable),
        "shouldn't respect unrewritable flag if looking at rules with nonexec attribute"
      );
      assert!(
        !flags.contains(StateFlag::RespectUnstackable),
        "shouldn't respect unstackable flag if looking at rules with nonexec attribute"
      );
    }

    if max_depth != PositionDepth::from_variant(PositionStateDepthSentinel::TopWithoutExtension) {
      flags.insert(StateFlag::WithExtension);
    }

    // SearchState uses the provided context for constructing
    // matches, resolving conditions and accumulating rewrite counts.

    let position_state = PositionState::new(
      context.get_root(),
      flags,
      min_depth,
      max_depth
    );

    Self {
      position_state,
      context,
      pre_equation          : None,
      substitution_variables: vec![],
      substitution_values   : RootVec::new(),
      matching_subproblem   : None,
      trial_ref             : RuleIndex::None,
      condition_stack       : vec![],
      rule_index            : RuleIndex::None,
    }
  }

  pub fn find_first_solution(
    &mut self,
    mut pre_eqn: PreEquationPtr,
    automaton: &mut dyn LHSAutomaton,
  ) -> bool {
    self.matching_subproblem = None;
    let subject = self.context.get_root();

    // Check component compatibility between pattern and subject
    if pre_eqn.lhs_term.kind() != Some(subject.symbol().range_kind()) {
      return false;
    }

    // Clear the rewriting context
    self.context.substitution.clear_first_n(pre_eqn.variable_info.protected_variable_count());

    // Attempt to initialize substitution
    if !self.init_substitution(&pre_eqn.variable_info) {
      return false;
    }

    // Attempt to match
    let maybe_extension_ptr = self.position_state.extension_info.as_mut().map(|extension_info| {UnsafePtr::new(extension_info)});
    let matched: bool;
    (matched, self.matching_subproblem) = automaton.match_(subject, &mut self.context.substitution, maybe_extension_ptr);
    if !matched {
      return false;
    }

    // If a subproblem was created, try to solve it
    if let Some(subproblem) = &mut self.matching_subproblem {
      if !subproblem.solve(true, &mut self.context) {
        return false;
      }
    }

    // Check the condition, if any
    if pre_eqn.has_condition() {
      let result =
          pre_eqn.check_condition_find_first(
            true,
            subject,
            &mut self.context,
            // This match statement is a testament to the stupidity of Rust's borrow checker.
            // This is what `as_deref_mut` is supposed to solve.
            match &mut self.matching_subproblem {
              None => {
                None
              }
              Some(reference) => {
                Some(reference.as_mut())
              }
            },
            &mut self.trial_ref,
            &mut self.condition_stack,
          );
      if !result {
        return false;
      }
    }
    self.pre_equation = Some(pre_eqn);
    true
  }

  pub fn find_next_solution(&mut self) -> bool {
    let Some(mut pre_eqn) = self.pre_equation else { return false };

    if pre_eqn.has_condition() {

      pre_eqn.check_condition_find_first(
        false,
        self.context.get_root(),
        self.context.as_mut(),
        // This match statement is a testament to the stupidity of Rust's borrow checker.
        // This is what `as_deref_mut` is supposed to solve.
        match &mut self.matching_subproblem {
          None => {
            None
          }
          Some(reference) => {
            Some(reference.as_mut())
          }
        },
        &mut self.trial_ref,
        &mut self.condition_stack,
      )
    } else {
      self.matching_subproblem
          .as_mut()
          .map_or(false, |sp| sp.solve(false, self.context.as_mut()))
    }
  }

  pub fn init_substitution(&mut self, var_info: &VariableInfo) -> bool {
    if self.substitution_variables.is_empty() {
      return var_info.unbound_variables.is_empty();
    }

    let nr_user_vars = self.substitution_variables.len();
    let nr_vars = var_info.real_variable_count();
    let mut bound = NatSet::new();

    for i in 0..nr_user_vars {
      let user_var = self.substitution_variables[i].as_ref();
      for j in 0..nr_vars {
        let j_var_index = VariableIndex::from_usize(j);
        if Some(user_var) == var_info.index_to_variable(j_var_index).as_deref() {
          self.context.substitution.bind(j_var_index, Some(self.substitution_values[i]));
          bound.insert(j);
          break;
        }
      }
    }

    bound.is_superset(&var_info.unbound_variables)
  }


}
