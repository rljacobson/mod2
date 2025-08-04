use crate::{
  api::DagNodePtr,
  core::{
    gc::root_container::{BxRootVec, RootVec},
    redex_position::RedexPosition,
    rewriting_context::{ContextAttribute, ContextAttributes},
    substitution::Substitution,
    VariableIndex,
    SentinelIndex
  }
};

pub type BxRewritingContext = Box<RewritingContext>;

pub struct RewritingContext {
  // "Base class" members
  pub root: Option<BxRootVec>,

  /// Statistics, records how many rewrites were done.
  pub membership_count       : u64,
  pub equation_count         : u64,
  pub rule_count             : u64,
  pub narrowing_count        : u64,
  pub variant_narrowing_count: u64,

  //	For rule rewriting
  // ToDo: These need to be marked!
  redex_stack  : Vec<RedexPosition>,
  redex_stack_roots: BxRootVec, // Solution to marking redex_stack
  stale_marker : VariableIndex, // NONE = -1, ROOT_OK = -2, an index when >= 0
  lazy_marker  : VariableIndex, // NONE = -1, an index when >= 0
  current_index: VariableIndex,

  pub substitution: Substitution,
  attributes      : ContextAttributes,

}

impl RewritingContext {
  pub fn new(root: Option<DagNodePtr>) -> BxRewritingContext {
    Box::new(
      RewritingContext {
        root                   : root.map(|r| RootVec::with_node(r)),
        membership_count       : 0,
        equation_count         : 0,
        rule_count             : 0,
        narrowing_count        : 0,
        variant_narrowing_count: 0,
        redex_stack            : vec![],
        redex_stack_roots      : RootVec::new(),
        stale_marker           : SentinelIndex::RootOk.into(),
        lazy_marker            : SentinelIndex::None.into(),
        current_index          : VariableIndex::Zero,
        substitution           : Substitution::default(),
        attributes             : ContextAttributes::default(),
      }
    )
  }

  //region Getters

  /// A limited RewritingContext:
  ///  1. Does not have a rootNode.
  ///  2. Need not have a substitution large enough to apply sort constraints.
  ///  3. ~Does not protect its substitution from garbage collection.~
  ///  4. ~Does not protect its redex stack from garbage collection.~
  /// It exists so that certain functions that expect a RewritingContext,
  /// ultimately to compute true sorts by applying sort constraints can be
  /// called by unification code when a general purpose RewritingContext
  /// not available. Sort constraints are not supported by unification and
  /// are thus ignored if the supplied RewritingContext is limited.
  #[inline(always)]
  pub fn is_limited(&self) -> bool {
    self.root.is_none()
  }

  #[inline(always)]
  pub fn finished(&mut self) {
    self.substitution.finished()
  }

  #[inline(always)]
  pub fn trace_abort(&self) -> bool {
    self.attributes.contains(ContextAttribute::Abort)
  }

  #[inline(always)]
  pub fn is_trace_enabled(&self) -> bool {
    self.attributes.contains(ContextAttribute::Trace)
  }

  /// Only for use when you know a root exists.
  #[inline(always)]
  pub fn get_root(&self) -> DagNodePtr {
    self.root.as_ref().unwrap().node()
  }

  // endregion Getters

  // region Statistics
  #[inline(always)]
  fn clear_counts(&mut self) {
    self.membership_count        = 0;
    self.equation_count          = 0;
    self.rule_count              = 0;
    self.narrowing_count         = 0;
    self.variant_narrowing_count = 0;
  }

  #[inline(always)]
  pub fn add_counts_from(&mut self, other: &RewritingContext) {
    self.membership_count        += other.membership_count;
    self.equation_count          += other.equation_count;
    self.rule_count              += other.rule_count;
    self.narrowing_count         += other.narrowing_count;
    self.variant_narrowing_count += other.variant_narrowing_count;
  }

  #[inline(always)]
  pub fn transfer_counts_from(&mut self, other: &mut RewritingContext) {
    self.add_counts_from(other);
    other.clear_counts();
  }

  // endregion

  // region Rebuilding Stale DagNodes

  fn rebuild_upto_root(&mut self) {
    let mut current_idx: VariableIndex;
    // println!("\nroot was {:?}", self.root_node);
    // println!("rebuilding from {}", self.current_index);
    // assert!(self.current_index >= 0, "bad currentIndex");

    // Locate deepest stack node with a stale parent.
    current_idx = self.current_index; // All staleness guaranteed to be above current_index
    while self.redex_stack[current_idx.idx()].parent_index != self.stale_marker {
      current_idx = self.redex_stack[current_idx.idx()].parent_index;
    }

    // We assume that we only have to rebuild the spine from staleMarker to root.
    let mut i: VariableIndex = self.stale_marker;

    while i != VariableIndex::None {
      self.remake_stale_dag_node(i, current_idx);
      current_idx = i;
      i = self.redex_stack[i.idx()].parent_index;
    }

    self.root = Some(RootVec::with_node(self.redex_stack[0].dag_node));
    self.stale_marker = SentinelIndex::RootOk.into();

    // println!("root is {:?}", self.root_node);
  }

  fn remake_stale_dag_node(&mut self, stale_index: VariableIndex, child_index: VariableIndex) {
    // Find first stacked argument of stale dag node.
    let mut first_idx = child_index.idx();
    while self.redex_stack[first_idx - 1].parent_index == stale_index {
      first_idx -= 1;
    }

    // Find last stacked argument of stale dag node.
    let mut last_idx = child_index.idx();
    let stack_length = self.redex_stack.len();
    while last_idx + 1 < stack_length && self.redex_stack[last_idx + 1].parent_index == stale_index {
      last_idx += 1;
    }

    // Replace stale dag node with a copy in which stacked arguments
    // replace corresponding arguments in the original.
    let remade = self.redex_stack[stale_index.idx() as usize]
      .dag_node
      .copy_with_replacements(&self.redex_stack, first_idx, last_idx);
    self.redex_stack[stale_index.idx() as usize].dag_node = remade;
  }

  // endregion

  #[inline(always)]
  pub fn reduce(&mut self) {
    if let Some(root) = &self.root {
      root.node().reduce(self);
    }
  }

}
