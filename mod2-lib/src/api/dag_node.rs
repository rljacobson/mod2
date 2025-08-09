/*!

The `DagNode` trait is the interface all DAG node's must implement.

Requirements of implementers of `DagNode`:
 1. DAG nodes should be newtypes of `DagNodeCore`. In particular...
 2. DAG nodes *must* have the same memory representation as a `DagNodeCore`.
 3. Implementers of `DagNode` are responsible for casting pointers, in particular its arguments.
 4. If an implementor owns resources, including ref counted objects like `IString`, it must provide an implementation
    of `DagNode::finalize()`. It must also set its `NeedsDestruction` flag.
 5. If an implementor holds no children, or if its children are represented differently than just `DagNodePtr`, it
    must provide an implementation of `iter_args`, `insert_child`, `len`, `compare_arguments`, `mark`.

*/

use std::{
  any::Any,
  cmp::{max, Ordering},
  fmt::Display,
  hash::{Hash, Hasher},
  iter::Iterator,
  ops::Deref,
  sync::atomic::Ordering::Relaxed,
};
use mod2_abs::{Outcome, UnsafePtr};
use crate::{
  api::{
    Arity,
    ExtensionInfo,
    MaybeExtensionInfo,
    MaybeSubproblem,
    SortCheckSubproblem,
    SymbolPtr,
    Term,
  },
  core::{
    dag_node_core::{
      DagNodeCore,
      DagNodeFlag,
      DagNodeFlags,
      ThinDagNodePtr
    },
    format::{
      FormatStyle,
      Formattable
    },
    gc::{
      node_allocator::ACTIVE_NODE_COUNT,
      gc_vector::{
        GCVector,
        GCVectorRefMut
      }
    },
    ArgIndex,
    DagNodeArguments,
    EquationalTheory,
    HashConsSet,
    RedexPosition,
    RedexPositionFlag,
    SortIndex,
    VariableIndex,
    rewriting_context::RewritingContext,
    sort::SortPtr,
    state_transition_graph::PositionIndex,
    substitution::Substitution,
  },
  impl_display_debug_for_formattable,
  HashType,
};

// A fat pointer to a trait object. For a thin pointer to a DagNodeCore, use ThinDagNodePtr
pub type DagNodePtr          = UnsafePtr<dyn DagNode + 'static>;
pub type MaybeDagNode        = Option<DagNodePtr>;
pub type DagNodeVector       = GCVector<DagNodePtr>;
pub type DagNodeVectorRefMut = GCVectorRefMut<DagNodePtr>;
pub type Multiplicity        = u8;

/// Commutative theories can have this more compact representation
#[derive(Copy, Clone)]
pub struct DagPair {
  pub(crate) dag_node    : DagNodePtr,
  pub(crate) multiplicity: Multiplicity,
}


pub trait DagNode {

  fn as_any(&self) -> &dyn Any;
  // {
  //   self
  // }

  fn as_any_mut(&mut self) -> &mut dyn Any;
  // {
  //   self
  // }

  fn as_ptr(&self) -> DagNodePtr;
  // {
  //   DagNodePtr::new(self as *const dyn DagNode as *mut dyn DagNode)
  // }

  /// Gives the same value as the corresponding method on `Term`, but computes it anew each time its called.
  /// This should be kept in sync with the theory's term structural hash computation.
  fn structural_hash(&self) -> HashType;

  // region Accessors

  /// Trait level access to members for shared implementation
  fn core(&self) -> &DagNodeCore;
  fn core_mut(&mut self) -> &mut DagNodeCore;

  #[inline(always)]
  fn arity(&self) -> Arity {
    self.core().arity()
  }

  /// MUST override if `Self::args` is not a [`DagNodeVector`].
  #[inline(always)]
  fn get_arguments(&self) -> DagNodeArguments {
    DagNodeArguments::from_node(self.as_ptr())
  }

  /// MUST override if `Self::args` is not a [`DagNodeVector`].
  ///
  /// Implement an empty iterator with:
  ///      `Box::new(std::iter::empty::<DagNodePtr>())`
  ///
  /// ToDo: Unify with `DagNodeArguments`.
  fn iter_args(&self) -> Box<dyn Iterator<Item=DagNodePtr>> {

    // The empty case
    if self.core().args.is_null() {
      assert!(
        self.arity().is_zero()
      );
      Box::new(std::iter::empty())
    }
    // The vector case
    // (other reasons for `needs_destruction` have null args)
    else if self.core().needs_destruction() {
      assert!( !self.symbol().is_variable() );
      assert!(
        self.arity().get() > 1,
        "Arity of node is {:?}", self.arity()
      );

      let node_vector: DagNodeVectorRefMut = arg_to_node_vec(self.core().args);
      Box::new(node_vector.iter().cloned())
    }
    // The singleton case
    else {
      assert!( !self.symbol().is_variable() );
      assert_eq!(self.arity().get(), 1);

      let node = arg_to_dag_node(self.core().args);

      // Make a fat pointer to the single node and return an iterator to it. This allows `self` to
      // escape the method. Of course, `self` actually points to a `DagNode` that is valid for the
      // lifetime of the program, so even in the event of the GC equivalent of a dangling pointer
      // or use after free, this will be safe.
      let v = unsafe { std::slice::from_raw_parts(&node, 1) };
      Box::new(v.iter().map(|n| *n))
    }
  }

  /// MUST override if `Self::args` is not a [`DagNodeVector`]
  fn insert_child(&mut self, new_child: DagNodePtr){
    // ToDo: Should we signal if arity is exceeded and/or DagNodeVector needs to reallocate?

    // Empty case
    if self.core().args.is_null() {
      self.core_mut().args = new_child.as_mut_ptr() as *mut u8;
    }
    // Vector case
    // (other reasons for `needs_destruction` have null args)
    else if self.core().needs_destruction() {
      let node_vec: DagNodeVectorRefMut = arg_to_node_vec(self.core_mut().args);
      node_vec.push(new_child)
    }
    // Singleton case
    else {
      let existing_child = arg_to_dag_node(self.core_mut().args);
      let arity          = max(self.arity().get()                       , 2);
      let node_vec       = DagNodeVector::with_capacity(arity as usize);

      node_vec.push(existing_child);
      node_vec.push(new_child);

      // Take ownership
      self.set_flags(DagNodeFlag::NeedsDestruction.into());
      self.core_mut().args = (node_vec as *mut DagNodeVector) as *mut u8;
    }
  }


  /// Gives the top symbol of this `DagNode`.
  #[inline(always)]
  fn symbol(&self) -> SymbolPtr {
    self.core().symbol
  }

  /// Gives the equational theory for this `DagNode`.
  #[inline(always)]
  fn theory(&self) -> EquationalTheory {
    self.symbol().theory()
  }

  #[inline(always)]
  fn sort(&self) -> Option<SortPtr> {
    let sort_index = self.sort_index();
    match sort_index {
      SortIndex::Unknown => None,

      // Anything else
      sort_index => {
        Some(self.symbol().range_kind().sorts[sort_index.get_unchecked() as usize])
      }
    }
  }


  #[inline(always)]
  fn set_sort_index(&mut self, sort_index: SortIndex) {
    self.core_mut().sort_index = sort_index;
  }


  #[inline(always)]
  fn sort_index(&self) -> SortIndex {
    self.core().sort_index
  }


  /// Set the sort to best of original and other sorts
  #[inline(always)]
  fn upgrade_sort_index(&mut self, other: DagNodePtr) {
    //  We set the sort to best of original and other sorts; that is:
    //    SORT_UNKNOWN, SORT_UNKNOWN -> SORT_UNKNOWN
    //    SORT_UNKNOWN, valid-sort -> valid-sort
    //    valid-sort, SORT_UNKNOWN -> valid-sort
    //    valid-sort,  valid-sort -> valid-sort
    match (self.sort_index(), other.sort_index()) {
      (SortIndex::Unknown, SortIndex::Unknown) => {
        self.set_sort_index(SortIndex::Unknown);
      }
      (SortIndex::Unknown, sort_index) => {
        self.set_sort_index(sort_index);
      }
      (sort_index, SortIndex::Unknown) => {
        self.set_sort_index(sort_index);
      }
      (sort_index, _sort_index_other) => {
        self.set_sort_index(sort_index);
      }
    }
  }


  /// MUST be overridden if `Self::args` is not a `DagNodeVec`
  fn len(&self) -> usize {
    // The empty case
    if self.core().args.is_null() {
      0

    } // The vector case
    else if self.core().needs_destruction() {
      // We need to allow `self` to escape the method, same as `Single(..)` branch.
      let node_vector: DagNodeVectorRefMut = arg_to_node_vec(self.core().args);

      node_vector.len()

    } // The singleton case
    else {
      1
    }
  }


  #[inline(always)]
  fn flags(&self) -> DagNodeFlags {
    self.core().flags
  }

  #[inline(always)]
  fn set_reduced(&mut self) {
    self.core_mut().flags.insert(DagNodeFlag::Reduced);
  }

  #[inline(always)]
  fn is_reduced(&self) -> bool {
    self.core().flags.contains(DagNodeFlag::Reduced)
  }

  #[inline(always)]
  fn is_unrewritable(&self) -> bool {
    self.core().flags.contains(DagNodeFlag::Unrewritable)
  }

  #[inline(always)]
  fn is_unstackable(&self) -> bool {
    self.core().flags.contains(DagNodeFlag::Unstackable)
  }

  #[inline(always)]
  fn set_unstackable(&mut self) {
    self.core_mut().flags.insert(DagNodeFlag::Unstackable)
  }

  #[inline(always)]
  fn is_copied(&self) -> bool {
    self.core().flags.contains(DagNodeFlag::Copied)
  }

  fn clear_copied_pointers(&mut self) {
    if self.is_copied() {
      let core = self.core_mut();
      core.flags.remove(DagNodeFlag::Copied);
      // ToDo: Uncomment if `DagNodeCore::symbol` and `DagNodeCore::forwarding_ptr` are ever unified.
      // if let Some(node) = core.forwarding_ptr {
      //   core.symbol = node.symbol();
      // }
      core.forwarding_ptr = None;
      self.clear_copied_pointers_aux();
    }
  }

  /// The theory specific part of `DagNode::clear_copied_pointers()`.
  fn clear_copied_pointers_aux(&mut self);

  #[inline(always)]
  fn set_flags(&mut self, flags: DagNodeFlags) {
    self.core_mut().flags.insert(flags);
  }

  // endregion Accessors

  // region Comparison

  /// Defines a partial order on `DagNode`s by comparing the symbols and the arguments recursively.
  fn compare(&self, other: DagNodePtr) -> Ordering {
    let symbol_order = self.symbol().compare(&*other.symbol());

    match symbol_order {
      Ordering::Equal => self.compare_arguments(other),
      _ => symbol_order,
    }
  }

  /// MUST be overridden is `Self::args` something other than a `DagNodeVector`.
  fn compare_arguments(&self, other: DagNodePtr) -> Ordering {
    let symbol = self.symbol();

    assert_eq!(symbol, other.symbol(), "symbols differ");

    if other.theory() != self.theory() {
      // if let None = other.as_any().downcast_ref::<FreeDagNode>() {}
      // Not even the same theory. It's not clear what to return in this case, so just compare symbols.
      return symbol.compare(&*other.symbol());
    };

    if (true, true) == (self.core().args.is_null(), other.core().args.is_null()) {
      return Ordering::Equal;
    }
    else if (false, false) == (self.core().args.is_null(), other.core().args.is_null()) {
      if (false, false) == (self.core().needs_destruction(), other.core().needs_destruction()) {
        // Singleton case
        let self_child     : DagNodePtr = arg_to_dag_node(self.core().args);
        let other_child_ptr: DagNodePtr = arg_to_dag_node(other.core().args);

        // Fast bail on equal pointers.
        if self_child.addr_eq(other_child_ptr){
          return Ordering::Equal; // Points to same node
        }

        return self_child.compare(other_child_ptr);
      }
      else if (true, true) == (self.core().needs_destruction(), other.core().needs_destruction()) {
        // The vector case
        let self_arg_vec : &DagNodeVector = arg_to_node_vec(self.core().args);
        let other_arg_vec: &DagNodeVector = arg_to_node_vec(other.core().args);

        // ToDo: This check isn't in Maude?
        if self_arg_vec.len() != other_arg_vec.len() {
          return if self_arg_vec.len() > other_arg_vec.len() {
            Ordering::Greater
          } else {
            Ordering::Less
          };
        }

        // Compare all children from left to right
        // Maude structures this so that it's tail call optimized, but we don't have that guarantee.
        for (&p, &q) in self_arg_vec.iter().zip(other_arg_vec.iter()) {
          // Fast bail on equal pointers.
          if p.addr_eq(q) {
            continue; // Points to same node
          }

          let result = p.compare(q);

          if result.is_ne() {
            return result;
          }
        }
      }
    }
    else {
      // It's not clear what to do in this case, if the case can even happen.
      return if other.core().args.is_null() {
        Ordering::Greater
      } else {
        Ordering::Less
      }
    }

    // Survived all attempts at finding inequality.
    Ordering::Equal
  }

  /// Checks pointer equality first, then compares symbols for equality recursively.
  fn equals(&self, other: DagNodePtr) -> bool {
    std::ptr::addr_eq(self, other.as_ptr())
      || (
      self.symbol() == other.symbol()
          && self.compare_arguments(other) == Ordering::Equal
      )
  }

  /// Tests whether `self`'s sort is less than or equal to other's sort
  fn leq_sort(&self, sort: SortPtr) -> bool {
    assert_ne!(self.sort_index(), SortIndex::Unknown, "unknown sort");
    self.sort().unwrap().leq(sort)
  }

  /// Only works for sorts on which `fast_geq_sufficient()` is true. Only used in
  /// `FreeRemainder::fast_match_and_replace()` and `FreeRemainder::*_check_and_bind()`.
  #[inline(always)]
  fn fast_leq_sort(&self, sort: SortPtr) -> bool {
    debug_assert!(sort.fast_geq_sufficient(), "invalid call to `DagNode::fast_leq_sort()`");
    sort.fast_geq(self.sort_index())
  }


  // endregion Comparison

  // region Copy Constructors

  /// For hash consing, recursively checks child nodes to determine if a canonical copy needs to be made.
  fn make_canonical(&self, hash_cons_set: &mut HashConsSet) -> DagNodePtr;

  /// For hash consing unreduced nodes, recursively creates a canonical copy.
  fn make_canonical_copy(&self, hash_cons_set: &mut HashConsSet) -> DagNodePtr;

  /// Makes a shallow clone of this node.
  fn make_clone(&self) -> DagNodePtr;

  /// Overwrites other with a clone of self. Invalidates existing fat pointers.
  /// MUST be overridden for nonstandard args or inline data that needs to be cloned.
  fn overwrite_with_clone(&mut self, mut other: DagNodePtr) -> DagNodePtr {
    let node_mut = other.core_mut();

    // Overwrite all `DagNodeCore` fields.
    node_mut.args       = self.shallow_copy_args();
    node_mut.inline     = self.core().inline;
    node_mut.symbol     = self.symbol();
    node_mut.sort_index = self.sort_index();
    // Copy over just the rewriting flags
    let rewrite_flags   = self.flags() & DagNodeFlag::RewritingFlags;
    node_mut.flags      = rewrite_flags;

    DagNodeCore::upgrade(node_mut)
  }

  fn copy_eager_upto_reduced(&mut self) -> MaybeDagNode {
    if self.is_reduced() {
      return None;
    }

    if !self.is_copied() {
      self.core_mut().forwarding_ptr = Some(self.copy_eager_upto_reduced_aux());
      self.set_flags(DagNodeFlag::Copied.into());
    }

    self.core_mut().forwarding_ptr
  }

  /// The implementor-specific part of `copy_eager_upto_reduced()`
  fn copy_eager_upto_reduced_aux(&mut self) -> DagNodePtr;

  /// A version of `copy_with_replacements` for a single replacement.
  fn copy_with_replacement(&self, arg_index: ArgIndex, replacement: DagNodePtr) -> DagNodePtr {
    let symbol = self.symbol();
    assert!(arg_index.is_index() && symbol.arity().get() > arg_index.get_unchecked(), "bad arg_index");

    let mut arguments = DagNodeArguments::from_args(self.shallow_copy_args(), self.arity());
    match &mut arguments {

      DagNodeArguments::Inline(args) => {
        *args = replacement;
      }

      DagNodeArguments::Vec(node_vec) => {
        node_vec[arg_index.idx()] = replacement;
      }

      _ => {
        // Should be impossible
        unreachable!();
      }
    }

    // Construct DAG node
    symbol.make_dag_node(arguments.as_args())
  }

  /// Creates a copy of the DAG node with multiple replacements using a redex stack.
  /// This is used during rewriting when specific subterms need to be replaced.
  fn copy_with_replacements(&self, redex_stack: &Vec<RedexPosition>, first_idx: usize, last_idx: usize) -> DagNodePtr {
    let mut arguments = DagNodeArguments::from_args(self.shallow_copy_args(), self.arity());

    match &mut arguments {

      DagNodeArguments::Inline(args) => {
        if redex_stack[first_idx].arg_index == 0 {
          // The only argument should be replaced.
          *args = redex_stack[first_idx].dag_node;
        }
      }

      DagNodeArguments::Vec(node_vec) => {
        for idx in first_idx..=last_idx.min((self.arity().get()-1) as usize) {
          node_vec[redex_stack[idx].arg_index.idx()] = redex_stack[redex_stack[idx].arg_index.idx()].dag_node;
        }
      }

      _ => {
        // Nothing to do.
      }
    }

    // Construct DAG node
    self.symbol().make_dag_node(arguments.as_args())
  }

  /// Only relevant for theories in which partial matching can occur, associative theories
  /// specifically. This method is called during rewriting when a rule matches only part of a term
  /// structure. It is used in rule application (`RuleTable::applyRules()`), configuration rewriting
  /// (`ConfigSymbol::leftOverRewrite()`), and position rebuilding (`PositionState::rebuildDag()`).
  fn partial_construct(&self, _replacement: DagNodePtr, _extension_info: &mut ExtensionInfo) -> DagNodePtr {
    unreachable!("Called on subject {}", self.symbol())
  }

  /// Used specifically in narrowing contexts to create a new DAG node
  /// by applying a substitution while replacing one specific argument.
  fn instantiate_with_replacement(
    &self,
    _substitution: &Substitution,
    _eager_copies: Option<&Vec<Option<DagNodePtr>>>,
    _arg_index   : ArgIndex,
    _new_dag_node: DagNodePtr
  ) -> DagNodePtr
  {
    unreachable!("Not implemented for theory {}", self.theory());
  }

  // endregion Copy Constructors

  // region GC related methods

  /// MUST override if `Self::args` is not a `DagNodeVector`.
  fn mark(&mut self) {
    if self.core().is_marked() {
      return;
    }

    ACTIVE_NODE_COUNT.fetch_add(1, Relaxed);
    self.core_mut().flags.insert(DagNodeFlag::Marked);

    // The empty case
    if self.core().args.is_null() {
      // pass
    } // The vector case
    else if self.core().needs_destruction() {
      {
        // Scope for mutable reference.
        let node_vector: DagNodeVectorRefMut = arg_to_node_vec(self.core().args);

        for node in node_vector.iter_mut() {
          node.mark();
        }
      }
      // Reallocate
      let node_vector: DagNodeVectorRefMut = arg_to_node_vec(self.core().args);
      self.core_mut().args = (node_vector.copy() as *mut DagNodeVector) as *mut u8;

    } // The singleton case
    else {
      // Guaranteed to be non-null.
      let mut node: DagNodePtr = arg_to_dag_node(self.core().args);
      node.mark();
    }
  } // end fn mark

  /// Returns a arg pointer pointing to either a DAG node vector or a single `DagNode`, or null if we have no
  /// arguments.
  fn shallow_copy_args(&self) -> *mut u8 {
    if !self.core().args.is_null() && self.core().needs_destruction() {
      // Reallocate
      let node_vector: DagNodeVectorRefMut = arg_to_node_vec(self.core().args);
      (node_vector.copy() as *mut DagNodeVector) as *mut u8
    } // The empty or singleton case
    else {
      // Be careful:
      self.core().args
    }
  }

  /// Finalize is run when this node is swept during garbage collection if its `NeedsDestruction` flag is set. The
  /// finalizer should only release whatever it is directly responsible for and cannot assume any of its children exist.
  fn finalize(&mut self) {
    /* empty default implementation */
  }

  // endregion GC related methods

  // region Compiler related methods

  /// Sets the sort_index of self. This is a method on Symbol in Maude.
  /// Called from `Symbol::fast_compute_true_sort()`, This virtual method
  /// determines the sort index for a DAG node based on its symbol type and
  /// the sorts of its arguments. Each symbol theory implements its own logic.
  fn compute_base_sort(&mut self);

  /// This version is designed for matching contexts where subproblems can be deferred.
  /// When sort checking fails but the symbol has sort constraints, it creates a
  /// `SortCheckSubproblem` for later resolution
  fn check_sort(&mut self, bound_sort: SortPtr) -> (Outcome, MaybeSubproblem) {
    if self.sort().is_some() {
      return (self.leq_sort(bound_sort).into(), None);
    }

    self.compute_base_sort();

    if self.leq_sort(bound_sort) {
      if !self.symbol().sort_constraint_free() {
        self.set_sort_index(SortIndex::Unknown);
      }
    } else {
      if self.symbol().sort_constraint_free() {
        return (Outcome::Failure, None);
      }
      self.set_sort_index(SortIndex::Unknown);
      let returned_subproblem = Box::new(SortCheckSubproblem::new(self.as_ptr(), bound_sort));
      return (Outcome::Success, Some(returned_subproblem))
    }

    (Outcome::Success, None)
  }

  /// This version is designed for rewriting contexts where sort constraints must be resolved immediately.
  fn check_sort_in_context(&mut self, bound_sort: SortPtr, context: &mut RewritingContext) -> Outcome{
    if self.sort_index() == SortIndex::Unknown {
      self.compute_base_sort();

      if self.leq_sort(bound_sort) {
        if !self.symbol().sort_constraint_free() {
          self.set_sort_index(SortIndex::Unknown);
        }
        return Outcome::Success;
      }

      if self.symbol().sort_constraint_free() {
        return Outcome::Failure;
      }

      let mut local = RewritingContext::new(Some(self.as_ptr()));
      self.symbol().constrain_to_smaller_sort(self.as_ptr(), &mut local);
      context.add_counts_from(&local);
    }

    self.leq_sort(bound_sort).into()
  }
  // endregion Compiler related methods

  // region Rewriting related methods

  /// Reduces this DAG node to its canonical form by applying equations and built-in operations.
  /// This is the primary entry point for equational rewriting - it repeatedly applies equations
  /// until no more reductions are possible. The method delegates to the symbol's `eqRewrite()`
  /// method to perform symbol-specific rewriting operations.
  fn reduce(&mut self, context: &mut RewritingContext) {
    while !self.is_reduced() {
      let mut symbol = self.symbol();

      if !symbol.rewrite(self.as_ptr(), context) {
        self.set_reduced();
        symbol.fast_compute_true_sort(self.as_ptr(), context);
      }
    }
  }

  /// Only implemented for associative theories and the `S_` theory.
  fn partial_replace(&mut self, _replacement: DagNodePtr, _extension_info: MaybeExtensionInfo) {
    unreachable!("partial_replace not implemented for this node type.")
  }

  /// This function needs to be defined for theories that only store and
  /// stack one copy of a repeated argument to avoid redundant rewrite steps.
  /// (This is a method on Symbol in Maude.)
  fn stack_physical_arguments(
    &mut self,
    stack              : &mut Vec<RedexPosition>,
    parent_index       : VariableIndex,
    respect_frozen     : bool,
    respect_unstackable: bool,
    is_eager_context   : bool,
  ) {
    // Default impl delegates to `self.stack_arguments()`, i.e. assumes
    // that physical arguments correspond to notional arguments.
    self.stack_arguments(
      stack,
      parent_index,
      respect_frozen,
      respect_unstackable,
      is_eager_context,
    )
  }

  /// This function must be defined for symbols that have arity > 0.
  fn stack_arguments(
    &mut self,
    _stack              : &mut Vec<RedexPosition>,
    _parent_index       : VariableIndex,
    _respect_frozen     : bool,
    _respect_unstackable: bool,
    _is_eager_context   : bool,
  ) {
    // Default version does nothing and can be used for symbols that have no arguments.
  }

  // endregion Rewriting related methods

  // region Matching related methods

  /// This method must be overridden in theories that need extension, namely associative theories.
  fn match_variable_with_extension(
    &self,
    _index         : VariableIndex,
    _sort          : SortPtr,
    _solution      : &mut Substitution,
    _extension_info: MaybeExtensionInfo
  ) -> (bool, MaybeSubproblem) {
    (false, None)
  }

  // endregion Matching related methods

}

// region trait impls for DagNode

// ToDo: Revisit whether `semantic_hash` is appropriate for the `Hash` trait.
// Use the `DagNode::structural_hash(…)` hash for `HashSet`s and friends.
impl Hash for dyn DagNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u32(self.structural_hash())
  }
}
// To use `DagNode` with `HashSet`, it needs to implement `Eq`
impl PartialEq for dyn DagNode {
  fn eq(&self, other: &Self) -> bool {
    self.structural_hash() == other.structural_hash()
  }
}
impl Eq for dyn DagNode {}

impl Formattable for dyn DagNode {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    match style {
      FormatStyle::Debug => {
        write!(f, "<{}, {:p}>", self.symbol(), self.as_ptr().as_ptr())?
      }

      _ => {
        write!(f, "<{}>", self.symbol())?
      }
    };

    if self.len() > 0 {
      let mut args = self.iter_args();
      write!(f, "(")?;
      args.next().unwrap().repr(f, style)?;
      for arg in args {
        write!(f, ", ")?;
        arg.repr(f, style)?;
      }
      write!(f, ")")?;
    }

    Ok(())

  }
}

impl_display_debug_for_formattable!(dyn DagNode);

// endregion trait impls for DagNode

// Unsafe free functions

/// Reinterprets `args` as a `DagNodePtr`. The caller MUST be sure
/// that `args` actually points to a `DagNode`.
#[inline(always)]
pub fn arg_to_dag_node(args: *mut u8) -> DagNodePtr {
  DagNodeCore::upgrade(args as ThinDagNodePtr)
}

/// Reinterprets `args` as a `DagNodeVectorRefMut`. The caller MUST
/// be sure that `args` actually points to a `DagNodeVector`.
#[inline(always)]
pub fn arg_to_node_vec(args: *mut u8) -> DagNodeVectorRefMut {
  unsafe { (args as *mut DagNodeVector).as_mut().unwrap() }
}

/// Reinterprets the `DagNodeVectorRefMut` as `*mut u8` to store in args.
#[inline(always)]
pub fn node_vec_to_args(node_vec: DagNodeVectorRefMut) -> *mut u8 {
  (node_vec as *mut DagNodeVector) as *mut u8
}
