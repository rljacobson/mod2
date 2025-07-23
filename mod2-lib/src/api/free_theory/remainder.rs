/*!

This is a variation of `FreeLHSAutomaton` for matching whatever is left of a pattern after free
symbols have been matched. (In Maude this isn't a subclass of `LhsAutomaton`.)

*/

use std::rc::Rc;
use mod2_abs::UnsafePtr;
use super::{BoundVariable, FreeVariable, GroundAlien, NonGroundAlien};
pub(crate) use crate::{
  api::{
    free_theory::{FreeOccurrence, FreeTerm},
    variable_theory::VariableTerm,
    BxLHSAutomaton,
    DagNodePtr,
    MaybeSubproblem,
    Subproblem,
    SubproblemSequence,
    Term,
  },
  core::{
    gc::ok_to_collect_garbage,
    rewriting_context::RewritingContext,
    format::{FormatStyle, Formattable},
    pre_equation::{
      PreEquation,
      PreEquationKind,
      PreEquationPtr,
      Equation,
    },
    ArgIndex,
    DagNodeArguments,
    SlotIndex,
    SortIndex,
    VariableIndex,
  }
};


pub type FreeRemainderPtr = UnsafePtr<FreeRemainder>;

/// There are potentially three ways to compute the remainder, two of which are shortcut
/// optimizations.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(i8)]
pub enum Speed {
  // > 0 super-fast; < 0 fast; = 0 slow
  Fast      = -1,
  Slow      =  0,
  SuperFast =  1,
}

pub struct FreeRemainder {
  //	To qualify for "fast" treatment the associated equation must:
  //	(1) have a lhs that parses into a non-error sort
  //	(2) have only free symbols in lhs
  //	(3) be left linear
  //	(4) be unconditional
  //	(5) have no "problem" variables (ones which need their bindings copied to avoid
  //	    eager evaluation of lazy subterm)
  //	(6) have the sort of each variable qualify with fastGeqSufficient()
  //	To qualify for "super-fast", additionally each variable must have a sort that
  //	is the unique maximal user sort in its component which must be error-free.
  /// > 0 super-fast; < 0 fast; = 0 slow
  pub(crate) fast    : Speed,
  /// remainder consists of a foreign equation that might collapse into free theory
  foreign            : bool,
  free_variables     : Vec<FreeVariable>,
  /// equation we are a remainder of
  pub(crate) equation: PreEquationPtr,
  bound_variables    : Vec<BoundVariable>,
  ground_aliens      : Vec<GroundAlien>,
  non_ground_aliens  : Vec<NonGroundAlien>,
}

impl FreeRemainder {
  /// Constructs a slow foreign remainder with the given equation.
  pub fn with_equation(equation: PreEquationPtr) -> Self {
    FreeRemainder {
      fast             : Speed::Slow,
      foreign          : true,
      free_variables   : vec![],
      equation,
      bound_variables  : vec![],
      ground_aliens    : vec![],
      non_ground_aliens: vec![],
    }
  }

  pub fn new(
    equation          : PreEquationPtr,
    free_symbols      : Vec<FreeOccurrence>,
    mut free_variables: Vec<FreeOccurrence>,
    bound_variables   : Vec<FreeOccurrence>,
    gnd_aliens        : Vec<FreeOccurrence>,
    non_gnd_aliens    : Vec<FreeOccurrence>,
    best_sequence     : Vec<ArgIndex>,
    mut sub_automata  : Vec<Option<BxLHSAutomaton>>,
    slot_translation  : &Vec<SlotIndex>,
  ) -> Self {
    // Preliminary determination of whether remainder will qualify for "fast" or "super-fast"
    // runtime treatment.
    let mut fast: Speed = if !equation.has_condition() {
      Speed::SuperFast
    } else {
      Speed::Slow
    };

    // Variables that will be unbound //
    let mut new_free_variables = free_variables
      .iter_mut()
      .map(|oc| {
        let parent            = free_symbols[oc.position.idx()].downcast_term::<FreeTerm>();
        assert!(parent.slot_index.is_index(), "bad slot for parent in equation");
        let parent_slot_index = parent.slot_index;
        let v                 = oc.downcast_term::<VariableTerm>();
        let sort              = v.sort().unwrap();

        if !sort.fast_geq_sufficient() {
          fast = Speed::Slow; // Need slow handling for full sort check
        } else {
          if fast == Speed::SuperFast {
            // Currently super-fast
            if !sort.error_free_maximal() {
              fast = Speed::Fast; // Downgrade to fast
            }
          }
        }

        FreeVariable {
          position : slot_translation[parent_slot_index.idx()],
          arg_index: oc.arg_index,
          var_index: v.index,
          sort     : Some(sort),
        }
      })
      .collect::<Vec<_>>();

    // Pseudo variables for left to right sharing //
    for oc in free_symbols.iter() {
      let free_term: &FreeTerm = oc.downcast_term::<FreeTerm>();
      if free_term.core().save_index != VariableIndex::None {
        let index  = free_term.core().save_index;
        let parent = free_symbols[oc.position.idx()].downcast_term::<FreeTerm>();
        // format!("bad slot for {} in {}", parent.repr(FormatStyle::Simple),
        // equation.repr(FormatStyle::Simple)).as_str()
        assert_ne!(parent.slot_index, SlotIndex::None, "bad slot for parent in equation");
        let new_free_var = FreeVariable {
          position : slot_translation[parent.slot_index.idx()],
          arg_index: oc.arg_index,
          var_index: index,
          sort     : Some(free_term.kind().unwrap().sort(SortIndex::new(0))),
        };
        new_free_variables.push(new_free_var);
      }
    }

    // Variables that will be bound //
    let new_bound_variables = bound_variables
      .iter()
      .map(|oc| {
        let parent   = free_symbols[oc.position.idx()].downcast_term::<FreeTerm>();
        // format!("bad slot for {} in {}", parent.repr(FormatStyle::Simple),
        // equation.repr(FormatStyle::Simple)).as_str()
        assert!(parent.slot_index.is_index(), "bad slot for parent in equation");
        let variable = oc.downcast_term::<VariableTerm>();
        fast         = Speed::Slow; // Need slow handling if there are nonlinear variables

        BoundVariable {
          position : slot_translation[parent.slot_index.idx()],
          arg_index: oc.arg_index,
          var_index: variable.index,
        }
      })
      .collect::<Vec<_>>();


    // Ground alien subterms //
    let ground_aliens = gnd_aliens
      .iter()
      .map(|oc| {
        let parent = free_symbols[oc.position.idx()].downcast_term::<FreeTerm>();
        fast       = Speed::Slow; // Need slow handling if there are nonlinear variables

        GroundAlien {
          position : parent.slot_index,
          arg_index: oc.arg_index,
          term     : oc.term,
        }
      })
      .collect::<Vec<_>>();

    // Non-ground alien subterms //
    let non_ground_aliens = best_sequence
      .into_iter()
      .map(|i| {
        let occurrence    = &non_gnd_aliens[i.idx()];
        let parent        = free_symbols[occurrence.position.idx()].downcast_term::<FreeTerm>();
        let mut automaton = None;
        fast              = Speed::Slow; // Need slow handling if there are nonlinear variables
        // Elements of `sub_automata` are consumed by replacing them with `None`.
        std::mem::swap(&mut sub_automata[i.idx()], &mut automaton);

        NonGroundAlien {
          position : slot_translation[parent.slot_index.idx()],
          arg_index: occurrence.arg_index,
          automaton: automaton.unwrap(),
        }
      })
      .collect::<Vec<_>>();

    FreeRemainder {
      fast,
      foreign        : false,
      free_variables : new_free_variables,
      equation,
      bound_variables: new_bound_variables,
      ground_aliens,
      non_ground_aliens,
    }
  }

  // region Rewriting related methods

  pub fn fast_match_replace(
    &mut self,
    subject: DagNodePtr,
    context: &mut RewritingContext,
    stack  : &mut Vec<DagNodeArguments>,
  ) -> bool {
    if !context.is_trace_enabled() {
      if self.fast == Speed::SuperFast {
        for var in &self.free_variables {
          let d = stack[var.position.idx()][var.arg_index.idx()];
          assert_ne!(d.sort_index(), SortIndex::Unknown, "missing sort information");
          context.substitution.bind(var.var_index, Some(d));
        }
      } else if self.fast == Speed::Fast {
        for var in &self.free_variables {
          let d = stack[var.position.idx()][var.arg_index.idx()];
          assert_ne!(d.sort_index(), SortIndex::Unknown, "missing sort information");
          if d.fast_leq_sort(var.sort.unwrap()) {
            context.substitution.bind(var.var_index, Some(d));
          } else {
            return false;
          }
        }
      } else {
        return self.slow_match_replace(subject, context, stack);
      }

      let Equation {rhs_builder, ..} = &self.equation.pe_kind
          else { unreachable!("called fast_match_replace on non-equation") };
      rhs_builder.replace(subject, &mut context.substitution);

      context.equation_count += 1;
      ok_to_collect_garbage();

      return true;
    }

    self.slow_match_replace(subject, context, stack)
  }

  fn slow_match_replace(
    &mut self,
    subject: DagNodePtr,
    context: &mut RewritingContext,
    stack  : &mut Vec<DagNodeArguments>,
  ) -> bool {
    context.substitution.clear_first_n(self.equation.variable_info.protected_variable_count());
    let r = self.slow_match_replace_aux(subject, context, stack);
    context.finished();
    ok_to_collect_garbage();
    r
  }

  pub fn slow_match_replace_aux(
    &mut self,
    subject: DagNodePtr,
    context: &mut RewritingContext,
    stack  : &mut Vec<DagNodeArguments>,
  ) -> bool {
    let mut maybe_subproblem: MaybeSubproblem = None;

    if self.foreign {
      let matched: bool;
      (matched, maybe_subproblem)
          = self.equation.lhs_automaton.as_mut().unwrap().match_(subject, &mut context.substitution, None);
      if !matched {
        return false;
      }

      if let Some(subproblem) = maybe_subproblem.as_mut() {
        if !subproblem.solve(true, context) {
          return false;
        }
      }
    } else {
      // Bind free variables
      for var in &self.free_variables {
        let dag_node = stack[var.position.idx()][var.arg_index.idx()];
        // ToDo: Is this unwrap always allowed?
        if !dag_node.leq_sort(var.sort.unwrap()) {
          return false;
        }
        context.substitution.bind(var.var_index, Some(dag_node));
      }

      // Check bound variables
      for var in &self.bound_variables {
        let expected = context.substitution.value(var.var_index);
        let actual   = stack[var.position.idx()][var.arg_index.idx()];
        if expected.is_some() && !actual.compare(expected.unwrap()).is_eq() {
          return false;
        }
      }

      // Match ground aliens
      for alien in &self.ground_aliens {
        let actual = stack[alien.position.idx()][alien.arg_index.idx()];
        if !alien.term.compare_dag_node(actual).is_eq() {
          return false;
        }
      }

      // Match non-ground aliens
      if !self.non_ground_aliens.is_empty() {
        debug_assert!(!self.non_ground_aliens.is_empty(), "no nonGroundAliens");
        let mut subproblems = SubproblemSequence::new();

        for alien in &mut self.non_ground_aliens {
          let dag_node = stack[alien.position.idx()][alien.arg_index.idx()];
          if let (true, maybe_subproblem) = alien.automaton.match_(dag_node, &mut context.substitution, None) {
            if let Some(subproblem) = maybe_subproblem {
              subproblems.add(subproblem);
            }
          } else {
            return false;
          }
        }

        if let Some(mut subproblem) = subproblems.extract_subproblem() {
          if !subproblem.solve(true, context) {
            return false;
          }
          maybe_subproblem = Some(subproblem); // retain it for later condition or trace
        }
      }
    }

    // Check condition
    if self.equation.has_condition() {
      // Save and restore stack if rewriting the condition clobbers it
      let mut saved_stack = vec![];
      std::mem::swap(&mut saved_stack, stack);
      let condition_passed =
          match maybe_subproblem {
            None => {
              self.equation.check_condition(subject, context, None)
            }
            Some(mut subproblem) => {
              self.equation.check_condition(subject, context, Some(subproblem.as_mut()))
            }
          };
      std::mem::swap(&mut saved_stack, stack); // restore stack

      if !condition_passed {
        return false;
      }
    }

    // Do replacement
    // let trace = context.is_trace_enabled();
    // ToDo: Implement tracing
    // if trace {
    //   context.trace_pre_eq_rewrite(subject, self.equation, RewriteKind::Normal);
    //   if context.trace_abort() {
    //     return false;
    //   }
    // }

    let Equation{ rhs_builder, ..} = &mut self.equation.pe_kind else { unreachable!() };
    rhs_builder.replace(subject, &mut context.substitution);
    context.equation_count += 1;

    // ToDo: Implement tracing
    // if trace {
    //   context.trace_post_eq_rewrite(subject);
    // }

    true
  }

  // endregion Rewriting related methods

}
