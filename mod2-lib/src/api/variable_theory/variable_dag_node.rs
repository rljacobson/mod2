/*!

`VariableDagNode`

# Memory

```ignore
DagNodeCore::args: null
DagNodeCore::inline: IString
```

Since `IString` is reference counted, it needs to be dropped in the destructor.


*/

use std::{
  any::Any,
  cmp::{
    max,
    Ordering
  },
  mem::transmute,
  ops::DerefMut
};

use mod2_abs::{
  as_bytes,
  debug,
  hash::hash2,
  IString
};
use crate::{
  api::{
    Arity,
    DagNode,
    DagNodePtr,
    DagNodeVector,
    DagNodeVectorRefMut,
    Symbol,
    SymbolPtr,
    arg_to_dag_node,
    arg_to_node_vec,
    variable_theory::VariableSymbol,
  },
  core::{
    dag_node_core::{
      DagNodeCore,
      DagNodeFlag,
      DagNodeFlags,
      ThinDagNodePtr
    },
    gc::allocate_dag_node,
    sort::SortIndex,
    EquationalTheory,
    HashConsSet,
    VariableIndex
  },
  HashType
};


/// The index into `DagNodeCode::inline` at which we store the `index` of `VariableDagNode`.
const VARIABLE_INDEX_OFFSET: usize = size_of::<IString>();

#[repr(transparent)]
pub struct VariableDagNode(DagNodeCore);

impl VariableDagNode {

  pub fn new(symbol: SymbolPtr, name: IString, index: VariableIndex) -> DagNodePtr {
    let mut node = DagNodeCore::new(symbol);
    {
      // Scope of this as `VariableDagNode`,
      let node_mut = node.as_any_mut().downcast_mut::<VariableDagNode>().unwrap();
      node_mut.set_name(name);
      node_mut.set_index(index);
    }
    // Needs destruction to drop the `IString` in `DagNodeCode::inline`, which decrements the `IString`'s internal
    // reference count.
    // ToDo: Decouple `DagNodeVector` ownership from `NeedsDestruction` flag. For now this is ok because
    //       `DagNodeCore.args` is null for all cases (so far) in which the `NeedsDestruction` flag is set but
    //       which have no argument vector.
    node.set_flags(DagNodeFlag::NeedsDestruction.into());

    node
  }

  pub fn name(&self) -> IString {
    // Reconstitute the `IString` from its raw bytes so we can clone it.
    let slice = &self.core().inline; // The name starts at offset 0.
    let name: IString = unsafe {
      std::ptr::read_unaligned(slice.as_ptr() as *const IString)
    };
    // Increments reference count
    let cloned_name = name.clone();
    // But don't run the destructor for the `IString` we own.
    std::mem::forget(name);

    cloned_name
  }

  // Store the raw bytes of `name` in `self.inline`; `name` is consumed and it's `drop` method is *not* called.
  fn set_name(&mut self, name: IString) {
    let base = self.core_mut().inline.as_mut_ptr().cast::<IString>();
    unsafe {
      std::ptr::write_unaligned(base, name);
    }
  }

  /// This refers to the variable index, the index within a `VariableInfo` instance.
  #[inline(always)]
  pub fn index(&self) -> VariableIndex {
    unsafe {
      let ptr = self.core().inline.as_ptr().add(VARIABLE_INDEX_OFFSET) as *const VariableIndex;
      std::ptr::read_unaligned(ptr)
    }
  }

  /// This refers to the variable index, the index within a `VariableInfo` instance.
  #[inline(always)]
  pub fn set_index(&mut self, index: VariableIndex) {
    unsafe {
      let ptr = self.core_mut().inline.as_mut_ptr().add(VARIABLE_INDEX_OFFSET) as *mut VariableIndex;
      std::ptr::write_unaligned(ptr, index);
    }
  }

}

impl DagNode for VariableDagNode {
  #[inline(always)]
  fn as_any(&self) -> &dyn Any {
    self
  }

  #[inline(always)]
  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  #[inline(always)]
  fn as_ptr(&self) -> DagNodePtr {
    DagNodePtr::new(self as *const dyn DagNode as *mut dyn DagNode)
  }

  fn structural_hash(&self) -> HashType {
    hash2(self.symbol().hash(), self.name().get_hash())
  }

  #[inline(always)]
  fn core(&self) -> &DagNodeCore {
    &self.0
  }

  #[inline(always)]
  fn core_mut(&mut self) -> &mut DagNodeCore {
    &mut self.0
  }

  fn iter_args(&self) -> Box<dyn Iterator<Item=DagNodePtr>> {
    Box::new(std::iter::empty::<DagNodePtr>())
  }

  fn clear_copied_pointers_aux(&mut self) {
    /* pass */
  }

  fn make_canonical(&self, _hash_cons_set: &mut HashConsSet) -> DagNodePtr {
    self.as_ptr()
  }

  fn make_canonical_copy(&self, _hash_cons_set: &mut HashConsSet) -> DagNodePtr {
    // In principle variable could rewrite to something else.
    self.make_clone()
  }

  fn make_clone(&self) -> DagNodePtr {
    let mut new_node  = VariableDagNode::new(self.symbol(), self.name(), self.index());
    // Copy over just the rewriting flags
    let rewrite_flags = self.flags() & DagNodeFlag::RewritingFlags;
    new_node.set_flags(rewrite_flags);
    new_node.set_sort_index(self.sort_index());

    new_node
  }

  fn copy_eager_upto_reduced_aux(&mut self) -> DagNodePtr {
    VariableDagNode::new(self.symbol(), self.name(), self.index())
  }

  fn finalize(&mut self) {
    #[cfg(feature = "gc_debug")]
    debug!(5, "Finalizing VariableDagNode");
    // Reconstitute the `IString` from its raw bytes so its destructor can be executed.
    let slice = &self.core().inline;
    let droppable_istring: IString = unsafe {
      std::ptr::read_unaligned(slice.as_ptr() as *const IString)
    };
    // Decrements reference count
    drop(droppable_istring);
  }

  fn compute_base_sort(&mut self) {
    if let Some(symbol) = self.symbol().as_any().downcast_ref::<VariableSymbol>() {
      let symbol_index = symbol.sort().index_within_kind;
      self.set_sort_index(symbol_index);
    } else {
      unreachable!("Failed to downcast to VariableSymbol. This is a bug.");
    }
  }
}
