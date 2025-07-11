/*!

Free theory automaton.

*/

// Free Theory
use super::super::{
  BoundVariable,
  FreeDagNode,
  FreeOccurrence
  ,
  FreeTerm,
  FreeVariable,
  GroundAlien,
  NonGroundAlien
};
use crate::{
  api::{
    BxLHSAutomaton,
    DagNode,
    DagNodePtr,
    LHSAutomaton,
    MaybeExtensionInfo,
    MaybeSubproblem,
    SubproblemSequence,
    SymbolPtr,
    Term,
    variable_theory::VariableTerm,
  },
  core::{
    ArgIndex,
    DagNodeArguments,
    SlotIndex,
    SortIndex,
    VariableIndex,
    substitution::Substitution,
  }
};


#[derive(Clone)]
pub struct FreeSubterm {
  position  : SlotIndex,
  arg_index : ArgIndex,
  symbol    : SymbolPtr,
  save_index: VariableIndex,
}


pub struct FreeLHSAutomaton {
  top_symbol: SymbolPtr,

  stack              : Vec<DagNodeArguments>,
  free_subterms      : Vec<FreeSubterm>,
  uncertain_variables: Vec<FreeVariable>,
  bound_variables    : Vec<BoundVariable>,
  ground_aliens      : Vec<GroundAlien>,
  non_ground_aliens  : Vec<NonGroundAlien>,
}

impl FreeLHSAutomaton {
  pub fn new(
    mut free_symbols: Vec<FreeOccurrence>,
    uncertain_vars  : Vec<FreeOccurrence>,
    bound_vars      : Vec<FreeOccurrence>,
    gnd_aliens      : Vec<FreeOccurrence>,
    non_gnd_aliens  : Vec<FreeOccurrence>,
    best_sequence   : Vec<ArgIndex>,
    mut sub_automata: Vec<Option<BxLHSAutomaton>>,
  ) -> Box<Self> {
    let free_symbol_count = free_symbols.len();
    let top_term          = free_symbols[0].downcast_term_mut::<FreeTerm>();
    let top_symbol        = top_term.symbol();
    let mut slot_count = SlotIndex::new(1);

    top_term.slot_index = SlotIndex::Zero;

    // Free symbol skeleton //
    // Start with 1, because 0th term is `top_term`, which we set above.
    let free_subterms = (1..free_symbol_count)
      .map(|i| {
        let oc_position       = free_symbols[i].position.idx();
        let oc_arg_index      =  free_symbols[i].arg_index;
        let parent_slot_index = {
          let parent: &FreeTerm = free_symbols[oc_position].downcast_term::<FreeTerm>();
          parent.slot_index
        };

        let term  : &mut FreeTerm = free_symbols[i].downcast_term_mut::<FreeTerm>();
        let symbol: SymbolPtr     = term.symbol();
        let free_subterm          = FreeSubterm {
          position  : parent_slot_index,
          arg_index : oc_arg_index,
          symbol    : symbol.clone(),
          save_index: term.core().save_index,
        };

        if symbol.arity().get() > 0 {
          term.slot_index = slot_count;
          slot_count += 1;
        }

        free_subterm
      })
      .collect::<Vec<_>>();

    let stack = Vec::with_capacity(slot_count.idx());

    // Variables that may be bound //

    let uncertain_variables = uncertain_vars
      .iter()
      .map(|occurrence| {
        let parent = free_symbols[occurrence.position.idx()].downcast_term::<FreeTerm>();
        let variable = occurrence.downcast_term::<VariableTerm>();
        FreeVariable {
          position:  parent.slot_index,
          arg_index: occurrence.arg_index,
          var_index: variable.index,
          sort:      variable.sort(),
        }
      })
      .collect::<Vec<_>>();

    // Variables that will be bound //

    let bound_variables = bound_vars
      .iter()
      .map(|occurrence| {
        let parent = free_symbols[occurrence.position.idx()].downcast_term::<FreeTerm>();
        let variable = occurrence.downcast_term::<VariableTerm>();
        BoundVariable {
          position:  parent.slot_index,
          arg_index: occurrence.arg_index,
          var_index: variable.index,
        }
      })
      .collect::<Vec<_>>();

    // Ground alien subterms //

    let ground_aliens = gnd_aliens
      .iter()
      .map(|oc| {
        let parent = free_symbols[oc.position.idx()].downcast_term::<FreeTerm>();
        GroundAlien {
          position : parent.slot_index,
          arg_index: oc.arg_index,
          term     : oc.term,
        }
      })
      .collect::<Vec<_>>();

    // Non-ground alien subterms //

    let non_ground_aliens = best_sequence
      .iter()
      .map(|&i| {
        let occurrence: &FreeOccurrence = &non_gnd_aliens[i.idx()];
        let parent = free_symbols[occurrence.position.idx()].downcast_term::<FreeTerm>();
        NonGroundAlien {
          position:  parent.slot_index,
          arg_index: occurrence.arg_index,
          automaton: sub_automata[i.idx()].take().unwrap(),
        }
      })
      .collect::<Vec<_>>();

    Box::new(FreeLHSAutomaton {
      top_symbol,
      stack,
      free_subterms,
      uncertain_variables,
      bound_variables,
      ground_aliens,
      non_ground_aliens,
    })
  }
}


impl LHSAutomaton for FreeLHSAutomaton {
  fn match_(
    &mut self,
    mut subject    : DagNodePtr,
    solution       : &mut Substitution,
    _extension_info: MaybeExtensionInfo
  ) -> (bool, MaybeSubproblem)
  {
    // ToDo: What variant of comparison should this be?
    if subject.symbol() != self.top_symbol {
      return (false, None);
    }

    if self.top_symbol.arity().is_zero() {
      return (true, None);
    }

    // Maude casts to a FreeDagNode?! Presumably because they want `match` to be a virtual function on the base class.
    if let Some(s) = subject.as_any_mut().downcast_mut::<FreeDagNode>() {
      self.stack[0] = s.get_arguments();

      let mut stack_idx: usize = 0;
      // Match free symbol skeleton.
      for i in &self.free_subterms {
        // It is important that this is _immutable_ access to the args list, because
        // a `SharedVec` is copy on write if the ref count is greater than 1.
        let d: DagNodePtr = self.stack[i.position.idx()][i.arg_index.idx()];
        if d.symbol() != i.symbol {
          return (false, None);
        }

        if i.save_index.is_index() {
          solution.bind(i.save_index, Some(d));
        }

        if !i.symbol.arity().is_zero() {
          stack_idx += 1;
          self.stack[stack_idx] = d.get_arguments();
        }
      }

      for i in &self.uncertain_variables {
        let d = self.stack[i.position.idx()][i.arg_index.idx()];
        let v = i.var_index;
        let b = solution.get(v);
        if b.is_none() {
          assert_ne!(
            d.sort_index(),
            SortIndex::Unknown,
            "missing sort information (2) for {:?}",
            d.symbol().name()
          );
          // ToDo: This unwrap might not be justified. If `i.sort.is_none()`, the condition is false.
          if i.sort.is_some() && d.leq_sort(i.sort.unwrap()) {
            solution.bind(v, Some(d));
          } else {
            return (false, None);
          }
        } else {
          if !d.eq(b.as_ref().unwrap()) {
            return (false, None);
          }
        }
      }

      for i in &self.bound_variables {
        if !self.stack[i.position.idx()][i.arg_index.idx()].eq(solution.get(i.var_index).as_ref().unwrap()) {
          return (false, None);
        }
      }

      for i in &self.ground_aliens {
        if i.term
          .compare_dag_node(self.stack[i.position.idx()][i.arg_index.idx()])
          .is_ne()
        {
          return (false, None);
        }
      }

      assert!(self.non_ground_aliens.len() > 0, "no nrNonGroundAliens");
      if !self.non_ground_aliens.is_empty() {
        let mut subproblems = SubproblemSequence::new();

        for i in &mut self.non_ground_aliens {
          if let (true, subproblem) = i.automaton.match_(
            self.stack[i.position.idx()][i.arg_index.idx()].clone(),
            solution,
            None
          ) {
            // Destructure `subproblem`
            if let Some(sp) = subproblem {
              subproblems.add(sp);
            }
          } else {
            return (false, None);
          }
        }
        return (true, subproblems.extract_subproblem());
      }
      (true, None)
    } else {
      panic!("FreeLHSAutomaton::match called with non Free DagNode. This is a bug.");
    }
  }
}
