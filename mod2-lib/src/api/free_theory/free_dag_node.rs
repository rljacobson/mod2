use std::{
  any::Any,
  cmp::{
    max,
    Ordering
  },
  ops::DerefMut
};

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
    Arity,
    dag_node::{
      DagNodeVector,
      DagNodeVectorRefMut,
      DagNode,
      DagNodePtr,
      arg_to_dag_node,
      arg_to_node_vec
    },
    symbol::SymbolPtr,
    term::Term,
  },
};

pub struct FreeDagNode(DagNodeCore);

impl FreeDagNode {

  pub fn new(symbol: SymbolPtr) -> DagNodePtr {
    DagNodeCore::with_theory(symbol, EquationalTheory::Free)
  }

  pub fn with_args(symbol: SymbolPtr, args: &mut Vec<DagNodePtr>) -> DagNodePtr {
    let mut node = DagNodeCore::with_theory(symbol, EquationalTheory::Free);

    node.set_flags(DagNodeFlag::NeedsDestruction.into());
    node.core_mut().args = (DagNodeVector::from_slice(args) as *mut DagNodeVector) as *mut u8;

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
  fn as_ptr(&self) -> DagNodePtr {
    DagNodePtr::new(self as *const dyn DagNode as *mut dyn DagNode)
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
