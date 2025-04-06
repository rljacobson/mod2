mod free_term;
mod free_dag_node;
mod free_symbol;

use std::ops::{Deref, DerefMut};
use crate::{
  core::sort::SortPtr,
  api::term::{Term, TermPtr}
};

pub use free_term::FreeTerm;
pub use free_dag_node::FreeDagNode;
pub use free_symbol::FreeSymbol;

// Small auxiliary types for the free theory

pub(crate) type FreeOccurrences = Vec<FreeOccurrence>;

/// A type erased term that exists under a free term that knows its position and arg index.
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct FreeOccurrence {
  position:  i32,
  arg_index: i32,
  term:      TermPtr,
}

impl FreeOccurrence {
  pub fn new(position: i32, arg_index: i32, term: TermPtr) -> Self {
    FreeOccurrence {
      position,
      arg_index,
      term,
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
  position:  i32,
  arg_index: i32,
  var_index: i32,
  sort:      SortPtr,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct BoundVariable {
  position:  i32,
  arg_index: i32,
  var_index: i32,
}


/// A `GroundAlien` happens to have the same structure as a `FreeOccurrence`.
pub(crate) type GroundAlien = FreeOccurrence;


// #[derive(Clone, PartialEq)]
// pub(crate) struct NonGroundAlien {
//   position:  i32,
//   arg_index: i32,
//   automaton: RcLHSAutomaton, //RefCell<dyn LHSAutomaton>,
// }
