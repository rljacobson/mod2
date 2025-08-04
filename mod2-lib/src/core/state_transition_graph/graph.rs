use std::collections::hash_map::Entry;
use std::collections::HashMap;
use crate::{
  api::DagNodePtr,
  core::{
    gc::ok_to_collect_garbage,
    state_transition_graph::{
      rewrite_search_state::RewriteSearchState,
      State,
      PositionDepth,
      PositionStateDepthSentinel,
      StateFlag
    },
    rewriting_context::{
      RewritingContext,
      BxRewritingContext
    },
    HashConsSet,
    StateGraphIndex,
  }
};

pub struct StateTransitionGraph {
  pub(crate) initial_context: BxRewritingContext,
  seen                      : Vec<State>,
  hash_cons_to_seen         : HashMap<DagNodePtr, StateGraphIndex>,
  hash_cons_set             : HashConsSet,
}

impl StateTransitionGraph {
  pub fn new(initial_context: BxRewritingContext) -> StateTransitionGraph {
    StateTransitionGraph{
      initial_context,
      seen             : Vec::new(),
      hash_cons_to_seen: HashMap::default(),
      hash_cons_set    : HashConsSet::new()
    }
  }

  pub fn state_count(&self) -> usize {
    self.seen.len()
  }

  pub fn get_state_dag(&self, state_idx: StateGraphIndex) -> DagNodePtr {
    // self.hash_cons_set.get_canonical(self.seen[state_idx.idx()].hash_cons_index)
    self.seen[state_idx.idx()].state_dag
  }

  pub fn get_next_state(&mut self, state_idx: StateGraphIndex, index: StateGraphIndex) -> StateGraphIndex {
    // Temporarily holds `self.seen[state_idx.idx()]`.
    let mut state: State;
    {
      // Get mutable access to the current state node using the provided state index.
      let dummy = &mut self.seen[state_idx.idx()];
      // A dummy state to temporarily swap with the state in `self.seen`, because we need mutable access
      // to `self.seen` elsewhere.
      state = State {
        state_dag     : dummy.state_dag, // an arbitrary value
        parent        : Default::default(),
        next_states   : vec![],
        rewrite_state : None,
        forward_arcs  : Default::default(),
        fully_explored: false,
      };
      std::mem::swap(dummy, &mut state);
    }

    // Fast path: If the requested index is already computed, return it directly.
    if index.idx() < state.next_states.len() {
      let next_state = state.next_states[index.idx()];
      // return the state to `self.seen`
      std::mem::swap(&mut state, &mut self.seen[state_idx.idx()]);
      return next_state;
    }

    // If the state has been fully explored and no more successors exist, return None.
    if state.fully_explored {
      // return the state to `self.seen`
      std::mem::swap(&mut state, &mut self.seen[state_idx.idx()]);
      return StateGraphIndex::None;
    }

    // If the rewrite state has not been initialized for this node, do it now.
    if state.rewrite_state.is_none() {
      // Get the canonical DAG representative for this state's hash-consed index.
      let canonical_dag = *self.hash_cons_set.get_canonical(state.state_dag).unwrap();

      // Create a new rewriting context for exploring rewrites from this DAG root.
      let new_context = RewritingContext::new(Some(canonical_dag));

      // Initialize the rewrite search state with appropriate behavior flags.
      state.rewrite_state = Some(RewriteSearchState::new(
        new_context,
        None, // No label filter
        StateFlag::GCContext
            | StateFlag::SetUnrewritable
            | StateFlag::RespectUnrewritable
            | StateFlag::SetUnstackable
            | StateFlag::RespectUnstackable,
        PositionDepth::Zero, // Minimum depth
        PositionDepth::from_variant(PositionStateDepthSentinel::Unbounded), // Unlimited depth
      ));
    }

    let rewrite_state = state.rewrite_state.as_mut().unwrap();
    // Get the rewrite_state context for tracing.
    // let context = &mut rewrite_state.context;

    // Attempt to generate enough successors to satisfy the requested index.
    while state.next_states.len() <= index.idx() {
      // Try to find the next applicable rewrite rule.
      let success = rewrite_state.find_next_rewrite();

      // Merge any rewrite statistics from the local context back to the global one.
      self.initial_context.transfer_counts_from(rewrite_state.context.as_mut());

      if success {
        let rule = rewrite_state.get_rule();

        // ToDo: Implement tracing.
        // let trace = context.is_trace_enabled();
        // if trace {
        //   context.trace_pre_rule_rewrite(rewrite_state.get_dag_node(), rule);
        //   if context.trace_abort() {
        //     // return state to self.seen
        //     return None;
        //   }
        // }

        // ToDo: Implement tracePreRuleRewrite() and traceAbort() if needed.

        // Apply the rule to generate a replacement DAG.
        let replacement    = rewrite_state.get_replacement();
        // Build a new DAG from the replacement and its enclosing context.
        let (rebuilt, _)   = rewrite_state.rebuild_dag(replacement);
        // Create a new rewriting context for the rewritten term.
        let mut subcontext = RewritingContext::new(Some(rebuilt));

        // Increment the rule application counter.
        self.initial_context.rule_count += 1;

        // ToDo: Implement trace.
        // if trace {
        //   subcontext.trace_post_rule_rewrite(rebuilt.clone());
        //   if subcontext.trace_abort() {
        //     // return state to self.seen
        //     return StateGraphIndex::None;
        //   }
        // }

        // Normalize the resulting term using equational reductions.
        subcontext.reduce();

        // If tracing determined we should abort, exit early.
        if subcontext.trace_abort() {
          // return the state to `self.seen`
          std::mem::swap(&mut state, &mut self.seen[state_idx.idx()]);
          return StateGraphIndex::None;
        }

        // Merge reduction statistics back into the global context.
        self.initial_context.add_counts_from(&subcontext);

        let canonical_dag = self.hash_cons_set.insert(rebuilt.clone());
        // Determine if this is a new state or previously seen one.
        let next_state = match self.hash_cons_to_seen.entry(canonical_dag){
          Entry::Occupied(entry) => {
            // Previously known state
            // self.hash_cons_to_seen[hash_cons_index]
            *entry.get()
          }
          Entry::Vacant(entry) => {
            // New state: allocate a new entry and update mappings.
            let new_idx = StateGraphIndex::from_usize(self.seen.len());
            entry.insert(new_idx);

            self.seen.push(State {
              state_dag: canonical_dag,
              parent: state_idx,
              next_states: vec![],
              rewrite_state: None,
              forward_arcs: HashMap::new(),
              fully_explored: false,
            });
            new_idx
          }
        };

        // Record the successor state and which rule caused the transition.
        state.next_states.push(next_state);
        state
            .forward_arcs
            .entry(next_state)
            .or_default()
            .insert(rule);

        // Allow garbage collection after a rewrite step.
        ok_to_collect_garbage();
      } else {
        // No more rewrites can be found: clean up and mark as complete.
        state.fully_explored = true;
        state.rewrite_state  = None;
        // return the state to `self.seen`
        std::mem::swap(&mut state, &mut self.seen[state_idx.idx()]);
        return StateGraphIndex::None;
      }
    }

    // We've produced enough successors: return the one at the requested index.
    let next_state = state.next_states[index.idx()];
    // return the state to `self.seen`
    std::mem::swap(&mut state, &mut self.seen[state_idx.idx()]);
    next_state
  }
}
