use std::{
  cmp::Ordering,
  any::Any,
  fmt::{Display, Formatter, Pointer}
};

use mod2_abs::{
  hash::hash2 as term_hash,
  NatSet
}
;
use crate::{
  api::{
    dag_node::{
      DagNode,
      DagNodeVector,
      DagNodePtr,
      arg_to_node_vec
    },
    term::{
      BxTerm,
      Term
    },
    symbol::SymbolPtr,
    free_theory::free_dag_node::FreeDagNode
  },
  core::{
    format::{
      FormatStyle,
      Formattable
    },
    term_core::TermCore,
    dag_node_core::{
      DagNodeCore,
      DagNodeFlag,
    },
    substitution::Substitution,
    VariableInfo
  }
};

pub struct FreeTerm{
  core                 : TermCore,
  pub(crate) args      : Vec<BxTerm>,
  pub(crate) slot_index: i32,
  pub(crate) visited   : bool,
}

impl FreeTerm {
  pub fn new(symbol: SymbolPtr) -> Self {
    Self {
      core      : TermCore::new(symbol),
      args      : vec![],
      slot_index: 0,
      visited   : false,
    }
  }
}

impl Display for FreeTerm {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    (self as &dyn Term).fmt(f)
  }
}

impl Formattable for FreeTerm {
  fn repr(&self, style: FormatStyle) -> String {
    let mut accumulator = String::new();
    match style {
      FormatStyle::Simple => {
        accumulator.push_str(self.symbol_ref().repr(style).as_str());
      }

      FormatStyle::Debug | _ => {
        accumulator.push_str(format!("free<{}>", self.symbol_ref().repr(style)).as_str());
      }
    }

    accumulator.push_str(format!("free<{}>", self.symbol_ref().repr(style)).as_str());
    if !self.args.is_empty() {
      accumulator.push('(');
      accumulator.push_str(
        self
            .args
            .iter()
            .map(|arg| arg.repr(style))
            .collect::<Vec<String>>()
            .join(", ").as_str()
      );
      accumulator.push(')');
    }

    accumulator
  }
}

// impl Display for FreeTerm {
//   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//     todo!()
//   }
// }

impl Term for FreeTerm {
  //region Representation and Reduction Methods
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn as_ptr(&self) -> *const dyn Term {
    self
  }

  /// In sync with `normalize`.
  fn semantic_hash(&self) -> u32 {
    let mut hash_value: u32 = self.symbol_ref().hash_value;

    for arg in &self.args {
      hash_value = term_hash(hash_value, arg.semantic_hash());
    }

    hash_value
  }

  /// In sync with `semantic_hash`
  fn normalize(&mut self, full: bool) -> (u32, bool) {
    let mut changed: bool = false;
    let mut hash_value: u32 = self.symbol_ref().hash_value;

    for arg in &mut self.args.iter_mut() {
      let (child_hash, child_changed): (u32, bool) = arg.normalize(full);
      // ToDo: It appears `full` is not used for the free theory. Is this true?
      // ToDo: The free theory never sets `changed=true`? Shouldn't we check against the cached hash or something?
      //       If so, why even have `normalize` in addition to `semantic_hash`?

      changed = changed || child_changed;
      hash_value = term_hash(hash_value, child_hash);
    }

    (hash_value, changed)
  }

  // endregion

  fn core(&self) -> &TermCore {
    &self.core
  }

  fn core_mut(&mut self) -> &mut TermCore {
    &mut self.core
  }

  fn iter_args(&self) -> Box<dyn Iterator<Item=&dyn Term> + '_> {
    Box::new(self.args.iter().map(|arg| arg.as_ref()))
  }

  // region Comparison Methods

  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering {
    assert!(&self.symbol_ref() == &other.symbol_ref(), "symbols differ");

    if let Some(other) = other.as_any().downcast_ref::<FreeTerm>() {
      for (arg_self, arg_other) in self.args.iter().zip(other.args.iter()) {
        let r = arg_self.compare(arg_other.as_ref());
        if r.is_ne() {
          return r;
        }
      }
      return Ordering::Equal;
    } else {
      unreachable!("Could not downcast Term to FreeTerm. This is a bug.")
    }
  }

  fn compare_dag_arguments(&self, other: &dyn DagNode) -> Ordering {
    // assert_eq!(self.symbol(), other.symbol(), "symbols differ");
    if let Some(other) = other.as_any().downcast_ref::<FreeDagNode>() {
      for (arg_self, arg_other) in self.args.iter().zip(other.iter_args()) {
        let arg_other: &dyn DagNode = unsafe { &*arg_other };
        let r = arg_self.compare_dag_node(arg_other);
        if r.is_ne() {
          return r;
        }
      }

      Ordering::Equal
    } else {
      unreachable!("Could not downcast Term to FreeTerm. This is a bug.")
    }
  }

  // ToDo: This method makes no use of partial_substitution except for `partial_compare_unstable` in `VariableTerm`.
  fn partial_compare_arguments(&self, partial_substitution: &mut Substitution, other: &dyn DagNode) -> Option<Ordering> {
    assert!(self.symbol_ref().compare(other.symbol_ref()).is_eq(), "symbols differ");

    for (term_arg, dag_arg) in self.iter_args().zip(other.iter_args()) {
      let r = term_arg.partial_compare(partial_substitution, unsafe{ &*dag_arg });
      if r?.is_ne() {
        return r;
      }
    }

    Some(Ordering::Equal)
  }

  // endregion

  fn dagify_aux(&self) -> DagNodePtr {
    let new_node = FreeDagNode::new(self.symbol());
    let new_node_ref = unsafe{ &mut *new_node };
    let args = arg_to_node_vec(new_node_ref.core().args);

    for arg in self.args.iter() {
      let node = arg.dagify();
      _ = args.push(node);
    }

    new_node
  }
/*
  // region Compiler-related
  #[inline(always)]
  fn compile_lhs(
    &self,
    match_at_top: bool,
    variable_info: &VariableInfo,
    bound_uniquely: &mut NatSet,
  ) -> (RcLHSAutomaton, bool) {
    FreeTerm::compile_lhs(self, match_at_top, variable_info, bound_uniquely)
  }

  /// The theory-dependent part of `compile_rhs` called by `term_compiler::compile_rhs(…)`. Returns
  /// the `save_index`.
  #[inline(always)]
  fn compile_rhs_aux(
    &mut self,
    rhs_builder: &mut RHSBuilder,
    variable_info: &VariableInfo,
    available_terms: &mut TermBag,
    eager_context: bool,
  ) -> i32 {
    FreeTerm::compile_rhs_aux(&mut self, rhs_builder, variable_info, available_terms, eager_context)
  }

  #[inline(always)]
  fn analyse_constraint_propagation(&mut self, bound_uniquely: &mut NatSet) {
    FreeTerm::analyse_constraint_propagation(self, bound_uniquely)
  }

  #[inline(always)]
  fn find_available_terms_aux(&self, available_terms: &mut TermBag, eager_context: bool, at_top: bool) {
    FreeTerm::find_available_terms_aux(&self, available_terms, eager_context, at_top);
  }
  // endregion
  */
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_free_term() {

  }
}
