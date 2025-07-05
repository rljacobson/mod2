mod free_term;
mod free_dag_node;
mod free_symbol;
mod free_automata;
// mod free_net;
// mod remainder;

use std::ops::{Deref, DerefMut};
use crate::{
  api::{
    term::{Term, TermPtr},
    automaton::BxLHSAutomaton
  },
  core::{
    sort::SortPtr,
    ArgIndex,
    SlotIndex,
    VariableIndex
  },
};

// Flatten module hierarchy
pub use free_term::FreeTerm;
pub use free_dag_node::FreeDagNode;
pub use free_symbol::FreeSymbol;

// Small auxiliary types for the free theory

pub(crate) type FreeOccurrences = Vec<FreeOccurrence>;

/// A type erased term that exists under a free term that knows its position and arg index.
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct FreeOccurrence {
  position:  SlotIndex,
  arg_index: ArgIndex,
  term:      TermPtr,
}

impl FreeOccurrence {
  pub fn new(position: SlotIndex, arg_index: ArgIndex, term: TermPtr) -> Self {
    FreeOccurrence {
      position,
      arg_index,
      term,
    }
  }

  /// Downcast the term to a mutable reference to a concrete term type. Panics if the term is not actually
  /// of type `T`.
  pub fn downcast_term<T: Term + 'static>(&self) -> &T {
    let term: &dyn Term = self.term.deref();

    if let Some(term) = term.as_any().downcast_ref() {
      term
    } else {
      unreachable!("Could not dereference as the requested type of Term. This is a bug.")
    }
  }

  /// Downcast the term to a mutable reference to a concrete term type. Panics if the term is not actually
  /// of type `T`.
  pub fn downcast_term_mut<T: Term + 'static>(&mut self) -> &mut T {
    let term: &mut dyn Term = self.term.deref_mut();

    if let Some(term) = term.as_any_mut().downcast_mut::<T>() {
      term
    } else {
      unreachable!("Could not dereference as the requested type of Term. This is a bug.")
    }
  }

  /// Downcast the term to a mutable reference to a concrete term type `T` if possible and returns a mutable reference.
  /// If the term is not of type `T`, returns `None`.
  pub fn try_downcast_term_mut<T: Term + 'static>(&mut self) -> Option<&mut T> {
    let term: &mut dyn Term = self.term.deref_mut();
    term.as_any_mut().downcast_mut::<T>()
  }

  pub fn term(&self) -> &dyn Term {
    self.term.deref()
  }

  pub fn term_mut(&mut self) -> &mut dyn Term {
    self.term.deref_mut()
  }
}

// These two structs are specific to the free theory. The ACU theory has its own version.
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct FreeVariable {
  position:  SlotIndex,
  arg_index: ArgIndex,
  var_index: VariableIndex,
  sort:      Option<SortPtr>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct BoundVariable {
  position:  SlotIndex,
  arg_index: ArgIndex,
  var_index: VariableIndex,
}


/// A `GroundAlien` happens to have the same structure as a `FreeOccurrence`.
pub(crate) type GroundAlien = FreeOccurrence;


// #[derive(Clone, PartialEq)]
pub(crate) struct NonGroundAlien {
  position:  SlotIndex,
  arg_index: ArgIndex,
  automaton: BxLHSAutomaton,
}


#[cfg(test)]
mod tests {
  use crate::{
    api::{
      Arity,
      dag_node_cache::DagNodeCache,
      symbol::Symbol
    },
    core::symbol::{SymbolAttributes, SymbolType}
  };
  use super::*;

  #[test]
  fn term_normalize_sets_hash(){
    let f = FreeSymbol::with_arity("f".into(), Arity::new_unchecked(2), None);
    let g = FreeSymbol::with_arity("g".into(), Arity::new_unchecked(2), None);
    let h = FreeSymbol::with_arity("h".into(), Arity::ZERO, None);

    // f(g(h, h), h)

    let mut h_term = FreeTerm::new(h.as_ptr(), vec![]);
    let h_term = match h_term.normalize(true) {
      (Some(new_h_term), changed, hash) => {
        println!("new h term: {}, changed: {}, hash: {}", new_h_term, changed, hash);
        assert_eq!(new_h_term.structural_hash(), hash);
        new_h_term
      }
      (None, true, hash) => {
        println!("h term changed: {}, hash: {}", h_term, hash);
        assert_eq!(h_term.structural_hash(), hash);
        Box::new(h_term)
      }
      (_, _, hash) => {
        println!("h term hash: {}", hash);
        assert_eq!(h_term.structural_hash(), hash);
        Box::new(h_term)
      }
    };

    let mut g_term = FreeTerm::new(g.as_ptr(), vec![h_term.deep_copy(), h_term.deep_copy()]);
    let g_term = match g_term.normalize(true) {
      (Some(new_g_term), changed, hash) => {
        println!("new g term: {}, changed: {}, hash: {}", new_g_term, changed, hash);
        assert_eq!(new_g_term.structural_hash(), hash);
        new_g_term
      }
      (None, true, hash) => {
        println!("g term changed: {}, hash: {}", g_term, hash);
        assert_eq!(g_term.structural_hash(), hash);
        Box::new(g_term)
      }
      (_, _, hash) => {
        println!("g term hash: {}", hash);
        assert_eq!(g_term.structural_hash(), hash);
        Box::new(g_term)
      }
    };

    let mut f_term = FreeTerm::new(f.as_ptr(), vec![g_term, h_term]);
    let _f_term = match f_term.normalize(true) {
      (Some(new_f_term), changed, hash) => {
        println!("new f term: {}, changed: {}, hash: {}", new_f_term, changed, hash);
        assert_eq!(new_f_term.structural_hash(), hash);
        new_f_term
      }
      (None, true, hash) => {
        println!("f term changed: {}, hash: {}", f_term, hash);
        assert_eq!(f_term.structural_hash(), hash);
        Box::new(f_term)
      }
      (_, _, hash) => {
        println!("f term hash: {}", hash);
        assert_eq!(f_term.structural_hash(), hash);
        Box::new(f_term)
      }
    };

  }

  #[test]
  fn test_dagify(){
    let f = FreeSymbol::with_arity("f".into(), Arity::new_unchecked(2), None);
    let g = FreeSymbol::with_arity("g".into(), Arity::new_unchecked(2), None);
    let h = FreeSymbol::with_arity("h".into(), Arity::ZERO, None);

    // f(g(h, h), h)
    let mut h_term = FreeTerm::new(h.as_ptr(), vec![]);
    let h_term = match h_term.normalize(true) {
      (Some(new_h_term), changed, hash) => {
        println!("new h term: {}, changed: {}, hash: {}", new_h_term, changed, hash);
        new_h_term
      }
      (None, true, hash) => {
        println!("h term changed: {}, hash: {}", h_term, hash);
        Box::new(h_term)
      }
      (_, _, hash) => {
        println!("h term hash: {}", hash);
        Box::new(h_term)
      }
    };

    let mut g_term = FreeTerm::new(g.as_ptr(), vec![h_term.deep_copy(), h_term.deep_copy()]);
    let g_term = match g_term.normalize(true) {
      (Some(new_g_term), changed, hash) => {
        println!("new g term: {}, changed: {}, hash: {}", new_g_term, changed, hash);
        assert_eq!(new_g_term.structural_hash(), hash);
        new_g_term
      }
      (None, true, hash) => {
        println!("g term changed: {}, hash: {}", g_term, hash);
        assert_eq!(g_term.structural_hash(), hash);
        Box::new(g_term)
      }
      (_, _, hash) => {
        println!("g term hash: {}", hash);
        assert_eq!(g_term.structural_hash(), hash);
        Box::new(g_term)
      }
    };

    let mut f_term = FreeTerm::new(f.as_ptr(), vec![g_term, h_term]);
    let f_term = match f_term.normalize(true) {
      (Some(new_f_term), changed, hash) => {
        println!("new f term: {}, changed: {}, hash: {}", new_f_term, changed, hash);
        assert_eq!(new_f_term.structural_hash(), hash);
        new_f_term
      }
      (None, true, hash) => {
        println!("f term changed: {}, hash: {}", f_term, hash);
        assert_eq!(f_term.structural_hash(), hash);
        Box::new(f_term)
      }
      (_, _, hash) => {
        println!("f term hash: {}", hash);
        assert_eq!(f_term.structural_hash(), hash);
        Box::new(f_term)
      }
    };

    println!("f_term: {}", f_term);

    let mut node_cache = DagNodeCache::new(false);
    let f_dag = f_term.dagify(&mut node_cache);
    println!("node_cache:\n[");
    for (k, v) in node_cache.map {
      println!("\t{}: {}", k, v);
    }
    println!("]");

    println!("f_dag (DEBUG): {:?}", f_dag);
    println!("f_dag: {}", f_dag);
  }
}
