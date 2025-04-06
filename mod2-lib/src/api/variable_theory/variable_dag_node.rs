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
  ops::DerefMut,
  mem::transmute
};

use mod2_abs::{as_bytes, debug, IString};

use crate::{
  core::{
    gc::{
      allocate_dag_node,
      increment_active_node_count
    },
    dag_node_core::{
      DagNodeCore,
      DagNodeFlags,
      DagNodeFlag,
      ThinDagNodePtr
    },
    EquationalTheory
  },
  api::{
    symbol::SymbolPtr,
    dag_node::{
      DagNodeVector,
      DagNodeVectorRefMut,
      DagNode,
      DagNodePtr,
      arg_to_dag_node,
      arg_to_node_vec
    },
    Arity
  },
};

pub struct VariableDagNode(DagNodeCore);

impl VariableDagNode {

  pub fn new(symbol: SymbolPtr, name: IString) -> DagNodePtr {
    let mut node = DagNodeCore::with_theory(symbol, EquationalTheory::Variable);

    // Needs destruction to drop the `IString` in `DagNodeCode::inline`, which decrements the `IString`'s internal
    // reference count.
    // ToDo: Decouple `DagNodeVector` ownership from `NeedsDestruction` flag.
    node.set_flags(DagNodeFlag::NeedsDestruction.into());

    // Store the raw bytes of `name` in `inline`
    node.core_mut()
        .inline[..size_of::<IString>()]
        .copy_from_slice(&as_bytes(&name));
    // Don't drop the `IString`
    std::mem::forget(name);

    node
  }

  pub fn name(&self) -> IString {
    // Reconstitute the `IString` from its raw bytes so we can clone it.
    let slice = &self.core().inline; //[..size_of::<IString>()];
    let name: IString = unsafe {
      std::ptr::read_unaligned(slice.as_ptr() as *const IString)
    };
    // Increments reference count
    let cloned_name = name.clone();
    // But don't run the destructor for the `IString` we own.
    std::mem::forget(name);

    cloned_name
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

  fn finalize(&mut self) {
    #[cfg(feature = "gc_debug")]
    debug!(5, "Finalizing VariableDagNode");
    // Reconstitute the `IString` from its raw bytes so its destructor can be executed.
    let slice = &self.core().inline[..size_of::<IString>()];
    let droppable_istring: IString = unsafe {
      std::ptr::read_unaligned(slice.as_ptr() as *const IString)
    };
    // Decrements reference count
    drop(droppable_istring);
  }
}
