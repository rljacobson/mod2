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
}
