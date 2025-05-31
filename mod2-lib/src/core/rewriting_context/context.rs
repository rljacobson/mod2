use crate::{
  core::{
    substitution::Substitution,
    rewriting_context::{ContextAttribute, ContextAttributes, Purpose},
    redex_position::RedexPosition,
    interpreter::InterpreterPtr,
    gc::root_container::{BxRootVec, RootVec}
  },
  api::dag_node::DagNodePtr,
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
  stale_marker : i32, // NONE = -1, ROOT_OK = -2, an index when >= 0
  lazy_marker  : i32, // NONE = -1, an index when >= 0
  current_index: i32,

  // "User Level" members
  parent          : Option<BxRewritingContext>,
  pub interpreter : InterpreterPtr,
  pub substitution: Substitution,
  purpose         : Purpose,
  trial_count     : usize,
  attributes      : ContextAttributes,

}

impl RewritingContext {
  pub fn new(root: Option<DagNodePtr>, interpreter: InterpreterPtr) -> BxRewritingContext {
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
        stale_marker           : 0,
        lazy_marker            : 0,
        current_index          : 0,
        parent                 : None,
        interpreter,
        substitution: Substitution::default(),
        purpose     : Purpose::TopLevelEval,
        trial_count : 0,
        attributes  : ContextAttributes::default(),
      }
    )
  }

  pub fn with_parent(
    root              : Option<BxRootVec>,
    parent            : Option<BxRewritingContext>,
    purpose           : Purpose,
    enable_local_trace: bool,
    interpreter       : InterpreterPtr,
  ) -> BxRewritingContext {
    Box::new(
      RewritingContext {
        root,
        equation_count         : 0,
        membership_count       : 0,
        narrowing_count        : 0,
        rule_count             : 0,
        variant_narrowing_count: 0,
        redex_stack            : vec![],
        redex_stack_roots      : RootVec::new(),
        stale_marker           : 0,
        lazy_marker            : 0,
        current_index          : 0,
        parent,
        interpreter,
        substitution: Default::default(),
        purpose,
        trial_count : 0,
        attributes  : if enable_local_trace {
          ContextAttribute::LocalTrace.into()
        } else {
          ContextAttributes::default()
        },
      }
    )
  }

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
  pub fn trace_abort(&self) -> bool {
    let attribute = ContextAttribute::Abort;
    self.attributes.contains(attribute)
  }

  #[inline(always)]
  pub fn is_trace_enabled(&self) -> bool {
    let attribute = ContextAttribute::Trace;
    self.attributes.contains(attribute)
  }

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
  fn transfer_counts_from(&mut self, other: &mut RewritingContext) {
    self.add_counts_from(other);
    other.clear_counts();
  }

  // endregion

/*
Rewriting code...

  // region Rebuilding Stale DagNodes

  fn rebuild_upto_root(&mut self) {
    let mut i: i32;
    let mut current_idx: i32;
    // println!("\nroot was {:?}", self.root_node);
    // println!("rebuilding from {}", self.current_index);
    assert!(self.current_index >= 0, "bad currentIndex");

    // Locate deepest stack node with a stale parent.
    current_idx = self.current_index; // All staleness guaranteed to be above current_index
    while self.redex_stack[current_idx as usize].parent_index != self.stale_marker {
      current_idx = self.redex_stack[current_idx as usize].parent_index;
    }

    // We assume that we only have to rebuild the spine from staleMarker to root.
    i = self.stale_marker;

    while i != UNDEFINED {
      self.remake_stale_dag_node(i, current_idx);
      current_idx = i;
      i = self.redex_stack[i as usize].parent_index;
    }

    self.root = Some(self.redex_stack[0].dag_node);
    self.stale_marker = ROOT_OK;

    // println!("root is {:?}", self.root_node);
  }

  fn remake_stale_dag_node(&mut self, stale_index: i32, child_index: i32) {
    // Find first stacked argument of stale dag node.
    let mut first_idx = child_index as usize;
    while self.redex_stack[first_idx - 1].parent_index == stale_index {
      first_idx -= 1;
    }

    // Find last stacked argument of stale dag node.
    let mut last_idx = child_index as usize;
    let stack_length = self.redex_stack.len();
    while last_idx + 1 < stack_length && self.redex_stack[last_idx + 1].parent_index == stale_index {
      last_idx += 1;
    }

    // Replace stale dag node with a copy in which stacked arguments
    // replace corresponding arguments in the original.
    let remade = self.redex_stack[stale_index as usize]
      .dag_node
      .copy_with_replacements(&self.redex_stack, first_idx, last_idx);
    self.redex_stack[stale_index as usize].dag_node = remade;
  }

  // endregion


  #[inline(always)]
  pub fn finished(&mut self) {
    self.substitution.finished()
  }

  #[inline(always)]
  pub fn reduce(&mut self) {
    if let Some(root) = &self.root {
      self.reduce_dag_node(root.node());
    }
  }

  #[inline(always)]
  pub fn reduce_dag_node(&mut self, mut dag_node: DagNodePtr) {
    while !dag_node.is_reduced() {
      let mut symbol = dag_node.symbol();

      if !(symbol.rewrite(dag_node, self)) {
        dag_node.set_reduced();
        self.fast_compute_true_sort(dag_node.clone());
      }
    }
  }

  /// Computes the true sort of root.
  #[inline(always)]
  fn fast_compute_true_sort(&mut self, dag_node: DagNodePtr) {
    // let root = self.root.unwrap();
    let t = dag_node.borrow().symbol().core().unique_sort_index;

    if t < 0 {
      dag_node.borrow_mut().compute_base_sort(); // usual case
    } else if t > 0 {
      dag_node.borrow_mut().set_sort_index(t); // unique sort case
    } else {
      self.slow_compute_true_sort(dag_node); // most general case
    }
  }

  /// Computes the true sort of root.
  fn slow_compute_true_sort(&mut self, dag_node: DagNodePtr) {
    // let root = self.root.unwrap();
    let mut symbol = dag_node.borrow_mut().symbol();
    symbol.sort_constraint_table()
          .constrain_to_smaller_sort(dag_node.clone(), self);
  }
  */
}