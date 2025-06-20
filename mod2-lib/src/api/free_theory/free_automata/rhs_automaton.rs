/*!

The RHS automaton for the free theory has six variations specialized for arities 0-3:

  * FreeFast3RHSAutomaton
  * FreeFast2RHSAutomaton
  * FreeTernaryRHSAutomaton
  * FreeBinaryRHSAutomaton
  * FreeUnaryRHSAutomaton
  * FreeNullaryRHSAutomaton

 */

use std::{cell::RefCell, rc::Rc};
use std::ops::DerefMut;
use crate::{
  core::{substitution::Substitution, VariableInfo},
  api::{
    dag_node::MaybeDagNode,
    free_theory::{
      free_automata::{
        // FreeBinaryRHSAutomaton,
        // FreeFast2RHSAutomaton,
        // FreeFast3RHSAutomaton,
        // FreeNullaryRHSAutomaton,
        FreeRHSAutomatonInstruction,
        // FreeTernaryRHSAutomaton,
        // FreeUnaryRHSAutomaton,
      },
      FreeDagNode,
    },
  },
};
use crate::api::automaton::RHSAutomaton;
use crate::api::dag_node::{arg_to_dag_node, arg_to_node_vec, DagNode, DagNodePtr, DagNodeVector, DagNodeVectorRefMut};
use crate::api::symbol::SymbolPtr;
use crate::core::dag_node_core::DagNodeCore;
use crate::core::VariableIndex;

#[derive(Default)]
pub struct FreeRHSAutomaton {
  instructions: Vec<FreeRHSAutomatonInstruction>,
}

impl FreeRHSAutomaton {
  fn new() -> Self {
    Self::default()
  }

  /// Constructs a new RHS automaton specialized to the provided arity and free variable count.
  pub fn with_arity_and_free_variable_count(_max_arity: u32, _free_variable_count: u32) -> Box<dyn RHSAutomaton> {
    // if max_arity > 3 {
      Box::new(FreeRHSAutomaton::new()) // general case
      /*
    } else {
      // We have six faster RHS automata for low arity cases.
      if free_variable_count > 1 {
        // Multiple low arity symbol cases.
        if max_arity == 3 {
          Box::new(FreeFast3RHSAutomaton::default()) // all dag nodes padded to 3 args
        } else {
          Box::new(FreeFast2RHSAutomaton::default()) // all dag nodes padded to 2 args
        }
      } else {
        // Single low arity symbol cases.
        if max_arity > 1 {
          if max_arity == 3 {
            Box::new(FreeTernaryRHSAutomaton::default())
          } else {
            Box::new(FreeBinaryRHSAutomaton::default())
          }
        } else {
          if max_arity == 1 {
            Box::new(FreeUnaryRHSAutomaton::default())
          } else {
            Box::new(FreeNullaryRHSAutomaton::default())
          }
        }
      }
    }
    */
  }

  pub fn add_free(&mut self, symbol: SymbolPtr, destination: VariableIndex, sources: &Vec<VariableIndex>) {
    let new_instruction = FreeRHSAutomatonInstruction {
      symbol,
      destination,
      sources: sources.clone(),
    };

    self.instructions.push(new_instruction);
  }

  fn fill_out_args(&self, instr: &FreeRHSAutomatonInstruction, matcher: &mut Substitution, dag_node: &mut dyn DagNode) {
    let arg_count = dag_node.len();
    // ToDo: Do something better than this pattern
    // The empty case
    if arg_count == 0 {
      // pass
    } // The vector case
    else if dag_node.core().needs_destruction() {
      // Scope for mutable reference.
      let node_vector: DagNodeVectorRefMut = arg_to_node_vec(dag_node.core().args);
      for j in 0..arg_count {
        let new_arg    = matcher.value(instr.sources[j]);
        // ToDo: Is this unwrap always justified?
        node_vector[j] = new_arg.unwrap();
      }
    } // The singleton case
    else {
      // Guaranteed to be non-null
      let new_node = matcher.value(instr.sources[0]);
      dag_node.core_mut().args = new_node.unwrap().as_ptr() as *mut u8;
    }
  }
}


impl RHSAutomaton for FreeRHSAutomaton {
  fn as_any(&self) -> &dyn std::any::Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
    self
  }

  fn remap_indices(&mut self, variable_info: &mut VariableInfo) {
    for instr in &mut self.instructions {
      instr.destination = variable_info.remap_index(instr.destination);
      for source in &mut instr.sources {
        *source = variable_info.remap_index(*source);
      }
    }
  }

  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode {
    let mut new_dag_node: MaybeDagNode = None;

    for i in self.instructions.iter() {
      let mut new_node = FreeDagNode::new(i.symbol.clone());
      self.fill_out_args(i, matcher, new_node.deref_mut());
      matcher.bind(i.destination, new_dag_node);

      new_dag_node = Some(new_node);
    }

    new_dag_node
  }

  fn replace(&mut self, mut old: DagNodePtr, matcher: &mut Substitution) -> DagNodePtr {
    let instruction_count = self.instructions.len();

    for instruction in &self.instructions[..instruction_count - 1] {
      let mut new_dag_node = FreeDagNode::new(instruction.symbol.clone());
      self.fill_out_args(instruction, matcher, new_dag_node.deref_mut());
      matcher.bind(instruction.destination, Some(new_dag_node));
    }

    // For the last instruction/node, we reuse to node that `old` points to.as
    // By doing this, we avoid having to call `matcher.bind` and updating
    // references to old.
    let instruction = &self.instructions[instruction_count - 1];
    let mut new_dag_node = FreeDagNode::new(instruction.symbol.clone());
    std::mem::swap(old.core_mut(), new_dag_node.core_mut());
    self.fill_out_args(instruction, matcher, old.deref_mut());

    DagNodeCore::upgrade(old.core_mut())
  }
}
