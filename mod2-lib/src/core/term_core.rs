/*!

A `Term` is a node in the expression tree. That is, an expression tree is a term, and
each subexpression is a term. 

The algorithms do not operate on expression trees (terms). Instead, the algorithms
operate on a directed acyclic graph (DAG) is constructed from the tree. Thus, for
each `Term` type, there is a corresponding `DagNode` type. However, because of
structural sharing, the node instances themselves are not in 1-to-1 correspondence.

*/

use std::{
  cell::Cell,
  ops::Deref
};

use enumflags2::{bitflags, BitFlags};

use mod2_abs::{
  NatSet,
  optimizable_int::OptU32
};
use crate::{
  api::{
    symbol::{
      SymbolSet,
      SymbolPtr,
      Symbol
    }
  },
  core::{
    sort::{
      kind::KindPtr,
      SortIndex
    },
    VariableIndex,
  },
  HashType,
};


#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TermType {
  Free,
  Bound,
  Ground,
  NonGround,
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TermAttribute {
  ///	A subterm is stable if its top symbol cannot change under instantiation.
  Stable,

  ///	A subterm is in an eager context if the path to its root contains only
  ///	eagerly evaluated positions.
  EagerContext,

  ///	A subterm "honors ground out match" if its matching algorithm guarantees
  ///	never to return a matching subproblem when all the terms variables
  ///	are already bound.
  HonorsGroundOutMatch,
}

pub type TermAttributes = BitFlags<TermAttribute, u8>;

#[derive(Clone)]
pub struct TermCore {
  /// The top symbol of the term
  pub(crate) symbol: SymbolPtr,
  /// The handles (indices) for the variable terms that occur in this term or its descendants
  pub(crate) occurs_set      : NatSet,
  pub(crate) context_set     : NatSet,
  // ToDo: How is this related to the kind of its symbol?
  pub(crate) kind            : Option<KindPtr>,
  pub(crate) collapse_symbols: SymbolSet,
  pub(crate) attributes      : TermAttributes,
  pub(crate) sort_index      : SortIndex,
  pub(crate) term_type       : TermType,
  pub(crate) save_index      : Option<VariableIndex>,
  /// Stores the structural hash computed in `Term::normalize()`
  pub(crate) hash_value      : HashType,

  /// The number of nodes in the term tree
  pub(crate) cached_size:  Cell<Option<OptU32>>,
}

impl TermCore {
  pub fn new(symbol: SymbolPtr) -> TermCore {
    TermCore {
      symbol,
      occurs_set      : Default::default(),
      context_set     : Default::default(),
      kind            : None,                 // ToDo: Initialize to `symbol.range_kind()` ?
      collapse_symbols: Default::default(),
      attributes      : TermAttributes::default(),
      sort_index      : SortIndex::default(),
      term_type       : TermType::Free,
      save_index      : None,
      hash_value      : 0,                    // Set in `Term::normalize()`
      cached_size     : Cell::new(None),
    }
  }

  // region Accessors

  /// Is the term stable?
  #[inline(always)]
  pub fn is_stable(&self) -> bool {
    self.attributes.contains(TermAttribute::Stable)
  }

  /// A subterm "honors ground out match" if its matching algorithm guarantees never to return a matching subproblem
  /// when all the terms variables are already bound.
  #[inline(always)]
  pub fn honors_ground_out_match(&self) -> bool {
    self.attributes.contains(TermAttribute::HonorsGroundOutMatch)
  }

  #[inline(always)]
  pub fn set_honors_ground_out_match(&mut self, value: bool) {
    if value {
      self.attributes.insert(TermAttribute::HonorsGroundOutMatch);
    } else {
      self.attributes.remove(TermAttribute::HonorsGroundOutMatch);
    }
  }

  #[inline(always)]
  pub fn is_eager_context(&self) -> bool {
    self.attributes.contains(TermAttribute::EagerContext)
  }

  #[inline(always)]
  pub fn is_variable(&self) -> bool {
    self.symbol.is_variable()
  }

  #[inline(always)]
  pub fn ground(&self) -> bool {
    self.occurs_set.is_empty()
  }

  /// The handles (indices) for the variable terms that occur in this term or its descendants, `occurs_set`.
  #[inline(always)]
  pub(crate) fn occurs_below(&self) -> &NatSet {
    &self.occurs_set
  }

  #[inline(always)]
  pub(crate) fn occurs_below_mut(&mut self) -> &mut NatSet {
    &mut self.occurs_set
  }

  #[inline(always)]
  pub(crate) fn occurs_in_context(&self) -> &NatSet {
    &self.context_set
  }

  #[inline(always)]
  pub(crate) fn occurs_in_context_mut(&mut self) -> &mut NatSet {
    &mut self.context_set
  }

  #[inline(always)]
  pub(crate) fn collapse_symbols(&self) -> &SymbolSet {
    &self.collapse_symbols
  }

  #[inline(always)]
  pub fn symbol(&self) -> SymbolPtr {
    self.symbol
  }

  #[inline(always)]
  pub fn symbol_ref(&self) -> &dyn Symbol {
    self.symbol.deref()
  }

  // endregion Accessors

}