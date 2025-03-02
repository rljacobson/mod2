use std::{
  any::Any,
  cmp::{
    max,
    Ordering
  }
};

use crate::{
  core::{
    allocator::{
      allocate_dag_node,
      increment_active_node_count
    },
    dag_node_core::{
      DagNodeCore,
      DagNodeFlags,
      DagNodeFlag,
      DagNodeTheory,
      ThinDagNodePtr
    }
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
  }
};

pub struct FreeDagNode(DagNodeCore);

impl FreeDagNode {

  pub fn new(symbol: SymbolPtr) -> DagNodePtr {
    assert!(!symbol.is_null());
    DagNodeCore::with_theory(symbol, DagNodeTheory::Free)
  }

  pub fn with_args(symbol: SymbolPtr, args: &mut Vec<DagNodePtr>) -> DagNodePtr {
    assert!(!symbol.is_null());
    let node     = DagNodeCore::with_theory(symbol, DagNodeTheory::Free);
    let node_mut = unsafe{ &mut *node };

    node_mut.set_flags(DagNodeFlag::NeedsDestruction.into());
    node_mut.core_mut().args = (DagNodeVector::from_slice(args) as *mut DagNodeVector) as *mut u8;

    node
  }

}

impl DagNode for FreeDagNode {
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

}
