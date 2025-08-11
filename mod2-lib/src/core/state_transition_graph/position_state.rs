use mod2_abs::debug;
use crate::{
  api::{
    DagNodePtr,
    ExtensionInfo
  },
  core::{
    state_transition_graph::{
      PositionDepth,
      PositionIndex,
      PositionStateDepthSentinel,
      StateFlag,
      StateFlags
    },
    ArgIndex,
    RedexPosition,
    VariableIndex,
  }
};
use crate::core::state_transition_graph::PositionStateSentinel;
use crate::core::substitution::Substitution;

pub struct PositionState {
  pub(crate) flags         : StateFlags,
  min_depth                : PositionDepth,
  max_depth                : PositionDepth,
  pub(crate) extension_info: Option<ExtensionInfo>,

  // For breadth-first traversal over positions
  position_queue : Vec<RedexPosition>,
  depth          : Vec<PositionDepth>,
  next_to_explore: PositionIndex,
  next_to_return : PositionIndex,
}

impl PositionState {
  pub fn new(
    top      : DagNodePtr,
    flags    : StateFlags,
    min_depth: PositionDepth,
    max_depth: PositionDepth
  ) -> Self
  {
    debug_assert!(
      !flags.contains(StateFlag::RespectUnstackable)
          || flags.contains(StateFlag::RespectFrozen),
      "can't respect unstackable if not respecting frozen otherwise we might miss frozen positions"
    );

    PositionState {
      flags,
      min_depth,
      max_depth,
      extension_info: None,

      // For breadth-first traversal over positions
      position_queue : vec![RedexPosition{
        dag_node    : top,
        parent_index: VariableIndex::None,
        arg_index   : ArgIndex::None,
        flags       : Default::default(),
      }],
      depth          : vec![PositionDepth::Zero],
      next_to_explore: PositionIndex::None,
      next_to_return : PositionIndex::None,
    }
  }

  pub fn position_index(&self) -> PositionIndex {
    self.next_to_return
  }

  pub fn explore_next_position(&mut self) -> bool {
    let finish = self.position_queue.len();

    loop {
      self.next_to_explore += 1;

      if self.next_to_explore.idx() >= finish {
        return false;
      }

      let our_depth = self.depth[self.next_to_explore.idx()];

      if our_depth >= self.max_depth {
        return false;
      }

      let redex_position = &self.position_queue[self.next_to_explore.idx()];
      let mut dag_node   = redex_position.dag_node;

      // Determine which flags to use
      let respect_frozen      = self.flags.contains(StateFlag::RespectFrozen);
      let respect_unstackable = self.flags.contains(StateFlag::RespectUnstackable);
      let is_eager            = redex_position.is_eager();

      dag_node.stack_physical_arguments(
        &mut self.position_queue,
        VariableIndex::from_usize(self.next_to_explore.idx()),
        respect_frozen,
        respect_unstackable,
        is_eager,
      );

      let new_finish = self.position_queue.len();

      if finish < new_finish {
        // Some new positions were added
        let new_depth = our_depth + 1;

        if self.depth.len() < new_finish {
          self.depth.resize(new_finish, new_depth);
        }

        for i in finish..new_finish {
          self.depth[i] = new_depth;
        }

        break;
      } else if self.flags.contains(StateFlag::SetUnstackable) && dag_node.is_unrewritable() {
        dag_node.set_unstackable();
      }
    }

    true
  }

  pub fn find_next_position(&mut self) -> bool {
    loop {
      self.next_to_return += 1;

      if self.next_to_return.idx() >= self.position_queue.len()
          && !self.explore_next_position() {
        return false;
      }

      // Skip positions shallower than the minimum depth
      if self.depth[self.next_to_return.idx()] >= self.min_depth {
        break;
      }
    }

    // If there's a maximum depth restriction, invalidate extension info.
    // This will force `make_extension_info()` if `get_extension_info()` is called.
    if !self.max_depth.is(PositionStateDepthSentinel::TopWithoutExtension) {
      self.flags.remove(StateFlag::ExtensionInfoValid);
      self.extension_info = None;
    }

    true
  }

  pub fn get_dag_node(&self) -> DagNodePtr {
    assert!(self.next_to_return.is_index(), "findNextPosition() not called");
    self.position_queue[self.next_to_return.idx()].dag_node
  }

  /// Rebuilds the dag node and returns a pair of dag nodes. The first dag is the
  /// rebuilt dag up to the root. The second dag is the replacement, possibly extended by
  /// extension information when only part of the redex is replaced (useful for tracing).
  pub fn rebuild_dag_with_extension(
    &mut self,
    mut replacement: DagNodePtr,
    extension_info : &mut Option<ExtensionInfo>,
    mut index      : PositionIndex,
  ) -> (DagNodePtr, DagNodePtr) {
    // If we matched only part of a subterm, use partialConstruct to extend the replacement.
    // Only relevant for associative theories.
    if let Some(info) = extension_info {
      if !info.matched_whole() {
        replacement = self.position_queue[index.idx()].dag_node.partial_construct(replacement.clone(), info);
      }
    }

    // Walk up the stack rebuilding
    let original_replacement = replacement.clone();
    let mut arg_index        = self.position_queue[index.idx()].arg_index;

    while let Some(i) = self.position_queue[index.idx()].parent_index.get() {
      let rp      = &self.position_queue[i as usize];
      replacement = rp.dag_node.copy_with_replacement(arg_index, replacement);
      arg_index   = rp.arg_index;
      index       = PositionIndex::new(i);
    }

    // Maude: We return the rebuilt dag, and the extended replacement term since the caller may
    // need the latter for tracing purposes.
    (replacement, original_replacement)
  }

  /// Rebuilds the dag node and returns a pair of dag nodes. The first dag is the
  /// rebuilt dag up to the root. The second dag is the replacement, possibly extended by
  /// extension information when only part of the redex is replaced (useful for tracing).
  #[inline(always)]
  pub(crate) fn rebuild_dag(&mut self, replacement: DagNodePtr) -> (DagNodePtr, DagNodePtr) {
    // We need mutable access to `extension_info`.
    let mut extension_info = self.extension_info.take();
    let result = self.rebuild_dag_with_extension(
        replacement,
        &mut extension_info,
        self.next_to_return
      );
    self.extension_info = extension_info;
    result
  }

  /// Rebuilds the DAG up to the root while **instantiating variables** along the path.
  ///
  /// Differences from `rebuild_dag_with_extension`:
  /// - **No extension support**: this is for narrowing; we assert that any extension info
  ///   was either absent or matched the whole redex in the original C++.
  /// - **Instantiation**: along the upward walk, we instantiate with a substitution, and
  ///   when the position is marked eager we provide eager copies of the substitution’s
  ///   bindings to avoid sharing dags across eager/lazy boundaries.
  ///
  /// Returns the rebuilt root DAG.
  pub fn rebuild_and_instantiate_dag(
    &self,
    replacement   : DagNodePtr,
    substitution  : &mut Substitution,
    first_variable: PositionIndex,
    last_variable : PositionIndex,
    mut index     : PositionIndex,
  ) -> DagNodePtr {
    // Extension is not supported for narrowing
    assert!(
      self.extension_info.is_none() || self.extension_info.as_ref().unwrap().matched_whole(),
      "Extension not supported"
    );

    // Walk up the stack rebuilding
    if index.is(PositionStateSentinel::None) {
      index = self.next_to_return;
    }
    // Start from the replacement term at the current redex position.
    let mut new_dag   = replacement;
    let mut arg_index = self.position_queue[index.idx()].arg_index;

    // Parent of the redex we’re rebuilding from.
    let mut parent = self.position_queue[index.idx()].parent_index.get();

    if parent.is_some() {

      // Maude: Make eager copies of bindings we will use to avoid sharing
      // dags that might rewrite between eager and lazy positions.
      debug!(5, "first_variable = {}  last_variable = {}", first_variable, last_variable);
      let mut eager_copies: Vec<Option<DagNodePtr>> = vec![None; last_variable.idx() + 1];

      for j in first_variable.idx()..=last_variable.idx() {
        // ToDo: Justify this unwrap
        let mut v = substitution.value(VariableIndex::from_usize(j)).unwrap();
        eager_copies[j] = v.copy_eager_upto_reduced();
      }

      for j in first_variable.idx()..=last_variable.idx() {
        substitution.value(VariableIndex::from_usize(j)).unwrap().clear_copied_pointers();
      }

      // Rebuild upwards, instantiating with eager/lazy bindings as appropriate
      while let Some(pidx) = parent {
        let redex_position = &self.position_queue[pidx as usize];

        // If the position is eager, pass the eager copies; otherwise pass no bindings (lazy).
        let bindings: Option<&Vec<Option<DagNodePtr>>> = if redex_position.is_eager() {
          Some(&eager_copies)
        } else {
          None
        };

        new_dag   = redex_position.dag_node.instantiate_with_replacement(substitution, bindings, arg_index, new_dag);
        arg_index = redex_position.arg_index;
        parent    = redex_position.parent_index.get();
      }
    }

    // Return the rebuilt DAG (root).
    new_dag
  }

}
