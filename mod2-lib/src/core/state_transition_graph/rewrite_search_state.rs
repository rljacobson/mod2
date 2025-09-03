/*!

In Maude, RewriteSearchState is a subclass of SearchState.

*/

use mod2_abs::{debug, IString, NatSet, UnsafePtr};
use crate::{
  core::{
    gc::root_container::{
      BxRootVec,
      RootVec
    },
    dag_node_core::DagNodeFlag,
    VariableInfo,
    VariableIndex,
    NoneSentinelIndex,
    RuleIndex,
    pre_equation::{
      RulePtr,
      PreEquationPtr,
      condition::ConditionState
    },
    rewriting_context::BxRewritingContext,
    state_transition_graph::{
      PositionDepth,
      PositionState,
      PositionStateDepthSentinel,
      StateFlag,
      StateFlags
    }
  },
  api::{
    LHSAutomaton,
    Subproblem,
    BxTerm,
    DagNodePtr,
  },
};


pub struct RewriteSearchState {
  position_state: PositionState,
  pub context   : BxRewritingContext,
  pre_equation  : Option<PreEquationPtr>,
  label         : Option<IString>,

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
    label    : Option<IString>,
    mut flags: StateFlags,
    min_depth: PositionDepth,
    max_depth: PositionDepth,
  ) -> Self {
    flags.insert(StateFlag::RespectFrozen);

    if label.is_some() {
      assert!(
        !flags.contains(StateFlag::SetUnrewritable),
        "shouldn't set unrewritable flag if only looking at rules with a given label"
      );
      assert!(
        !flags.contains(StateFlag::SetUnstackable),
        "shouldn't set unstackable flag if only looking at rules with a given label"
      );
    }
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

    // `SearchState` uses the provided context for constructing
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
      label,
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

  pub fn find_next_rewrite(&mut self) -> bool {
    let state_flags = self.position_state.flags;
    let label = self.label.clone();
    let mut rewrite_seen_at_current_position = false;

    if !self.rule_index.is(NoneSentinelIndex::None) {
      if self.find_next_solution() {
        return true;
      }
      rewrite_seen_at_current_position = true;
    } else {
      if !self.position_state.find_next_position() {
        return false;
      }
    }

    self.rule_index += 1;
    let allow_nonexec = state_flags.contains(StateFlag::AllowNonexec);

    loop {
      let mut dag_node = self.position_state.get_dag_node();

      let respect_unrewritable = state_flags.contains(StateFlag::RespectUnrewritable);
      let set_unrewritable     = state_flags.contains(StateFlag::SetUnrewritable);

      if !(respect_unrewritable && dag_node.is_unrewritable()) {
        let symbol = dag_node.symbol();
        let rules  = symbol.rules();

        while self.rule_index.idx() < rules.len() {
          // ToDo: Get rid of this.
          let rule_ptr = rules[self.rule_index.idx()];
          let mut rule = rules[self.rule_index.idx()];

          if (allow_nonexec || !rule.is_nonexec())
              && (label.is_none() || rule.label == label)
          {
            debug!(
                4,
                "trying rule {:?} at position {} dagNode {:?}",
                rule,
                self.position_state.position_index(),
                dag_node
            );

            let lhs_automaton = if self.position_state.flags.contains(StateFlag::WithExtension) {
              rule.get_ext_lhs_automaton()
            } else {
              rule.get_non_ext_lhs_automaton()
            };

            if self.find_first_solution(rule_ptr, lhs_automaton) {
              return true;
            }
          }

          self.rule_index += 1;
        }

        if !rewrite_seen_at_current_position && set_unrewritable {
          dag_node.core_mut().flags.insert(DagNodeFlag::Unrewritable);
        }
      }

      rewrite_seen_at_current_position = false;
      self.rule_index = RuleIndex::Zero;

      if !self.position_state.find_next_position() {
        break;
      }
    }

    false
  }

  pub fn get_rule(&self) -> RulePtr {
    let node   = self.position_state.get_dag_node();
    let symbol = node.symbol();
    let rules  = symbol.rules();
    rules[self.rule_index.idx()]
  }

  pub fn get_replacement(&mut self) -> DagNodePtr {
    let rule = self.get_rule();
    let rhs_builder = rule.get_rhs_builder();
    rhs_builder.construct(&mut self.context.substitution)
  }

  pub fn rebuild_dag(&mut self, dag_node: DagNodePtr) -> (DagNodePtr, DagNodePtr) {
    self.position_state.rebuild_dag(dag_node)
  }
}
