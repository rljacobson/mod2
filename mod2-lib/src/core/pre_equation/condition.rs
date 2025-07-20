/*!

Equations, rules, membership axioms, and strategies can have optional conditions that
must be satisfied in order for the pre-equation to apply. Conditions are like a
"lite" version of `PreEquation`.

In Maude, conditions are called condition fragments; the conjunction of all condition
fragments is called the condition. Maude also uses distinct subclasses of the base
class `ConditionFragment`. We use an enum.

We call a single condition "fragment" just a `Condition`, and we call the
conjunction of `Condition`s just `Conditions` (plural). Instead of subclasses,
`Condition` is an enum with variants for `Equality`, `SortMembership`
("sort test" in Maude), `Match` ("assignment" in Maude), and `Rewrite`.

*/

use std::{
  fmt::Display,
  ops::Deref
};
use mod2_abs::NatSet;
use crate::{
  api::{
    BxLHSAutomaton,
    BxTerm,
    LHSAutomaton,
    MaybeDagNode,
    MaybeSubproblem,
    Subproblem,
    TermPtr,
  },
  core::{
    automata::RHSBuilder,
    rewriting_context::RewritingContext,
    sort::SortPtr,
    state_transition_graph::StateTransitionGraph,
    substitution::Substitution,
    StateGraphIndex,
    TermBag,
    VariableIndex,
    VariableInfo,
  },
};
use Condition::*;

pub type Conditions  = Vec<BxCondition>;
pub type BxCondition = Box<Condition>;

// Also called Assignment
pub struct MatchConditionState {
  saved      : Substitution,
  rhs_context: Box<RewritingContext>,
  subproblem : Option<Box<dyn Subproblem>>,
  succeeded  : bool,
}

/// Creates a new `ConditionState::Match`.
///
/// Note that this does not retain a reference to the provided matcher.
impl MatchConditionState {
  pub fn new(
    original    : &mut RewritingContext,
    matcher     : &mut dyn LHSAutomaton,
    rhs_instance: MaybeDagNode
  ) -> Self {
    let mut saved = Substitution::with_capacity(original.substitution.fragile_binding_count());
    saved.copy_from_substitution(&original.substitution);

    let mut rhs_context = RewritingContext::new(rhs_instance);
    rhs_context.reduce();
    original.add_counts_from(rhs_context.as_ref());

    let (succeeded, subproblem) = matcher.match_(rhs_context.root.as_ref().unwrap().node(), &mut original.substitution, None);

    MatchConditionState {
      saved,
      rhs_context,
      subproblem,
      succeeded,
    }
  }

  pub fn solve(&mut self, find_first: bool, solution: &mut RewritingContext) -> bool {
    if self.succeeded {
      if let Some(subproblem) = self.subproblem.as_mut() {
        if subproblem.solve(find_first, solution) {
          return true;
        }
      } else {
        if find_first {
          return true;
        }
      }
    }
    solution.substitution.copy_from_substitution(&self.saved);

    false
  }
}

pub struct RewriteConditionState {
  state_graph: StateTransitionGraph,
  // matcher    : Option<BxLHSAutomaton>,
  saved      : Substitution,
  subproblem : MaybeSubproblem,
  explore    : StateGraphIndex,
  edge_count : StateGraphIndex,
}

impl RewriteConditionState {
  /// Creates a new `ConditionState::Rewrite`
  pub fn new(
    original    : &mut RewritingContext,
    lhs_instance: MaybeDagNode,
    // matcher     : Option<BxLHSAutomaton>,
  ) -> Self
  {
    let state_graph = StateTransitionGraph::new(RewritingContext::new(lhs_instance));

    let mut saved = Substitution::with_capacity(original.substitution.fragile_binding_count());
    saved.copy_from_substitution(&original.substitution);

    RewriteConditionState {
      state_graph,
      saved,
      subproblem: None,
      explore   : StateGraphIndex::None,
      edge_count: StateGraphIndex::None,
    }
  }

  pub fn solve(
    &mut self,
    find_first : bool,
    solution   : &mut RewritingContext,
    rhs_matcher: &mut dyn LHSAutomaton
  ) -> bool
  {
    if !find_first {
      if let Some(ref mut sp) = self.subproblem.as_mut() {
        if sp.solve(false, solution) {
          return true;
        }
        self.subproblem = None;
      }
      solution.substitution.copy_from_substitution(&self.saved);
    }

    loop {
      let state_nr = self.find_next_state();
      solution.add_counts_from(&self.state_graph.initial_context);

      if state_nr == StateGraphIndex::None {
        break;
      }

      let dag = self.state_graph.get_state_dag(state_nr);
      let success;

      (success, self.subproblem) = rhs_matcher.match_(dag, &mut solution.substitution, None);

      if success {
        if self.subproblem
               .as_mut()
               .map_or(true, |sp| sp.solve(true, solution))
        {
          return true;
        }
        self.subproblem = None;
      }

      solution.substitution.copy_from_substitution(&self.saved);
    }

    false
  }

  fn find_next_state(&mut self) -> StateGraphIndex {
    if self.explore == StateGraphIndex::None {
      self.explore = StateGraphIndex::Zero;
      return StateGraphIndex::Zero;
    }

    let state_count = self.state_graph.state_count();

    while self.explore.idx() < state_count {
      loop {
        self.edge_count += 1;
        let state_nr = self.state_graph.get_next_state(self.explore, self.edge_count);

        if state_nr == StateGraphIndex::None {
          if self.state_graph.initial_context.trace_abort() {
            return StateGraphIndex::None;
          }
          break; // try next explore state
        }

        if state_nr.idx() == state_count {
          return state_nr;
        }
      }

      self.edge_count = StateGraphIndex::None;
      self.explore += 1;
    }

    StateGraphIndex::None
  }
}

/// Holds state information used in solving condition fragments.
pub enum ConditionState {
  // Also called Assignment
  Match(MatchConditionState),
  Rewrite(RewriteConditionState),
}

impl ConditionState {
  pub fn solve(
    &mut self,
    find_first : bool,
    solution   : &mut RewritingContext,
    mut matcher: Option<&mut dyn LHSAutomaton>
  ) -> bool
  {
    match self {
      ConditionState::Match(match_state)      => match_state.solve(find_first, solution),
      ConditionState::Rewrite(rewrite_state) => rewrite_state.solve(find_first, solution, matcher.as_deref_mut().unwrap()),
    }
  }
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

  // region Compiler related methods


  pub fn preprocess(&mut self) {
    match self {

      Match { lhs_term, rhs_term, .. } => {
        lhs_term.fill_in_sort_info();
        rhs_term.fill_in_sort_info();
        assert_eq!(lhs_term.kind(), rhs_term.kind(), "component clash");
        lhs_term.analyse_collapses()
      }

      Equality { lhs_term, rhs_term, .. } => {
        lhs_term.fill_in_sort_info();
        rhs_term.fill_in_sort_info();
        assert_eq!(lhs_term.kind(), rhs_term.kind(), "component clash");
      }

      Rewrite { lhs_term, rhs_term, .. } => {
        lhs_term.fill_in_sort_info();
        rhs_term.fill_in_sort_info();
        assert_eq!(lhs_term.kind(), rhs_term.kind(), "component clash");
        rhs_term.analyse_collapses()
      }

      SortMembership { lhs_term, sort, .. } => {
        lhs_term.fill_in_sort_info();
        assert_eq!(lhs_term.kind(), sort.kind, "component clash");
      }

    }
  }

  pub fn compile_build(&mut self, variable_info: &mut VariableInfo, available_terms: &mut TermBag) {
    match self {
      Equality {
        lhs_term,
        rhs_term,
        lhs_index,
        rhs_index,
        builder,
        ..
      } => {
        *lhs_index = lhs_term.compile_rhs(builder, variable_info, available_terms, true);
        *rhs_index = rhs_term.compile_rhs(builder, variable_info, available_terms, true);
        variable_info.use_index(*lhs_index);
        variable_info.use_index(*rhs_index);
        variable_info.end_of_fragment();
      }

      SortMembership {
        lhs_term,
        lhs_index,
        builder,
        ..
      } => {
        *lhs_index = lhs_term.compile_rhs(builder, variable_info, available_terms, true);
        variable_info.use_index(*lhs_index);
        variable_info.end_of_fragment();
      }

      Match {
        lhs_term,
        rhs_term,
        rhs_index,
        builder,
        ..
      } => {
        *rhs_index = rhs_term.compile_rhs(builder, variable_info, available_terms, true);
        variable_info.use_index(*rhs_index);

        lhs_term.find_available_terms(available_terms, true, false);

        let lhs_term = lhs_term;
        lhs_term.determine_context_variables();
        lhs_term.insert_abstraction_variables(variable_info);
        variable_info.end_of_fragment();
      }

      Rewrite {
        lhs_term,
        rhs_term,
        lhs_index,
        builder,
        ..
      } => {
        // ToDo: Why call `compile_rhs` on the lhs term?
        *lhs_index = lhs_term.compile_rhs(builder, variable_info, available_terms, true);
        variable_info.use_index(*lhs_index);

        rhs_term.find_available_terms(available_terms, true, false);

        rhs_term.determine_context_variables();
        rhs_term.insert_abstraction_variables(variable_info);
        variable_info.end_of_fragment();
      }
    }
  }

  pub fn compile_match(&mut self, variable_info: &mut VariableInfo, bound_uniquely: &mut NatSet) {
    match self {
      Equality {
        lhs_index,
        rhs_index,
        builder,
        ..
      } => {
        builder.remap_indices(variable_info);
        *lhs_index = variable_info.remap_index(*lhs_index);
        *rhs_index = variable_info.remap_index(*rhs_index);
      }

      SortMembership { lhs_index, builder, .. } => {
        builder.remap_indices(variable_info);
        *lhs_index = variable_info.remap_index(*lhs_index);
      }

      Match {
        lhs_term,
        rhs_index,
        lhs_matcher,
        builder,
        ..
      } => {
        builder.remap_indices(variable_info);
        *rhs_index = variable_info.remap_index(*rhs_index);

        let (new_matcher, _subproblem_likely): (BxLHSAutomaton, bool) =
            lhs_term.compile_lhs(false, variable_info, bound_uniquely);
        *lhs_matcher = Some(new_matcher);

        bound_uniquely.union_in_place(lhs_term.occurs_below())
      }

      Rewrite {
        rhs_term,
        lhs_index,
        rhs_matcher,
        builder,
        ..
      } => {
        builder.remap_indices(variable_info);
        *lhs_index = variable_info.remap_index(*lhs_index);

        let (new_matcher, _subproblem_likely): (BxLHSAutomaton, bool) =
            rhs_term.compile_lhs(false, variable_info, bound_uniquely);
        *rhs_matcher = Some(new_matcher);

        bound_uniquely.union_in_place(rhs_term.occurs_below())
      }
    }
  }

  // endregion Compiler related methods

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

  pub fn solve(&mut self, find_first: bool, solution: &mut RewritingContext, state: &mut Vec<ConditionState>) -> bool {
    match self {
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
        let lhs_root        = solution.substitution.get(*lhs_index);
        let mut lhs_context = RewritingContext::new(lhs_root);
        let rhs_root        = solution.substitution.get(*rhs_index);
        let mut rhs_context = RewritingContext::new(rhs_root);

        lhs_context.reduce();
        solution.add_counts_from(&lhs_context);
        rhs_context.reduce();
        solution.add_counts_from(&rhs_context);

        (*lhs_context.root.unwrap().as_ref()).deref() == (*rhs_context.root.unwrap().as_ref()).deref()
      }

      SortMembership {
        builder,
        lhs_index,
        sort,
        ..
      } => {
        if !find_first {
          return false;
        }

        builder.safe_construct(&mut solution.substitution);
        let lhs_root        = solution.substitution.get(*lhs_index);
        let mut lhs_context = RewritingContext::new(lhs_root);

        lhs_context.reduce();
        solution.add_counts_from(&lhs_context);

        lhs_context
            .root
            .as_ref()
            .unwrap()
            .node()
            .leq_sort(*sort)
      }

      // `Match` is also called `Assignment` in Maude
      Match {
        builder,
        lhs_matcher,
        rhs_index,
        ..
      } => {
        if find_first {
          builder.safe_construct(&mut solution.substitution);
          let rhs_value = solution.substitution.get(*rhs_index);
          let mut condition_state
              = MatchConditionState::new(solution, lhs_matcher.as_deref_mut().unwrap(), rhs_value);

          if condition_state.solve(true, solution) {
            state.push(ConditionState::Match(condition_state));
            return true;
          }
        } else {
          let Some(mut condition_state) = state.pop() else {
            panic!("Expected ConditionState::Match on the stack");
          };

          if condition_state.solve(false, solution, None) {
            state.push(condition_state);
            return true;
          }
        }
        false
      }

      Rewrite {
        builder,
        lhs_index,
        rhs_matcher,
        ..
      } => {
        if find_first {
          builder.safe_construct(&mut solution.substitution);
          let lhs_value = solution.substitution.get(*lhs_index);
          let mut condition_state = RewriteConditionState::new(solution, lhs_value);

          if condition_state.solve(true, solution, rhs_matcher.as_deref_mut().unwrap()) {
            state.push(ConditionState::Rewrite(condition_state));
            return true;
          }
        } else {
          let Some(ConditionState::Rewrite(mut rewrite_condition_state)) = state.pop() else {
            panic!("Expected RewriteConditionState on the stack");
          };

          if rewrite_condition_state.solve(false, solution, rhs_matcher.as_deref_mut().unwrap()) {
            state.push(ConditionState::Rewrite(rewrite_condition_state));
            return true;
          }
        }
        false
      }
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
