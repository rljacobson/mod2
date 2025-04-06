use std::{
  any::Any,
  cmp::Ordering
};

use rand::seq::index::IndexVec;

use mod2_abs::{IString, NatSet, hash::hash2, PartialOrdering};

use crate::{
  core::{
    format::{FormatStyle, Formattable},
    term_core::TermCore,
    TermBag,
    substitution::Substitution
  },
  api::{
    variable_theory::VariableType,
    term::{Term, TermPtr, BxTerm},
    symbol::SymbolPtr,
    dag_node::{DagNode, DagNodePtr},
  },
  impl_display_debug_for_formattable,
  HashType,
  UNDEFINED,
};
use crate::api::variable_theory::VariableDagNode;

#[derive(Clone)]
pub struct VariableTerm {
  pub name         : IString,
  pub index        : i32,           // Set in `Term::index_variables()`
  pub variable_type: VariableType,
  pub core         : TermCore,
}

impl VariableTerm {
  pub fn new(name: IString, symbol: SymbolPtr) -> Self {
    VariableTerm{
      name,
      index        : UNDEFINED,     // Set in `Term::index_variables()`
      variable_type: VariableType::Blank,
      core         : TermCore::new(symbol)
    }
  }
}

impl Term for VariableTerm {
  #[inline(always)]
  fn as_any(&self) -> &dyn Any {
    self
  }

  #[inline(always)]
  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  #[inline(always)]
  fn as_ptr(&self) -> TermPtr {
    TermPtr::new(self as *const dyn Term as *mut dyn Term)
  }

  fn copy(&self) -> BxTerm {
    Box::new(self.clone())
  }

  #[inline(always)]
  fn hash(&self) -> HashType {
    self.symbol().hash()
  }

  /// Returns a pointer to the normalized version of self. If a new term was created during
  /// normalization, it is returned. We also need to know if any subterm changed, so we also
  /// return a bool, and unless the term is the expression's top-most term, we will always need
  /// the new hash value, too. The returned tuple is thus `( Option<TermBx>, changed, new_hash)`.
  ///
  /// Note: The hash value of a term is first set in this method.
  fn normalize(&mut self, _full: bool) -> (Option<BxTerm>, bool, HashType) {
    let hash_value = hash2(self.symbol().hash(), self.name.get_hash());
    self.core_mut().hash_value = hash_value;
    
    (None, false, hash_value)
  }

  #[inline(always)]
  fn core(&self) -> &TermCore {
    &self.core
  }

  #[inline(always)]
  fn core_mut(&mut self) -> &mut TermCore {
    &mut self.core
  }

  #[inline(always)]
  fn iter_args(&self) -> Box<dyn Iterator<Item=TermPtr> + '_> {
    Box::new(std::iter::empty::<TermPtr>())
  }

  #[inline(always)]
  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering {
    self.core.symbol.name().cmp(&other.symbol().name())
  }

  #[inline(always)]
  fn compare_dag_arguments(&self, other: &dyn DagNode) -> Ordering {
    if let Some(other) = other.as_any().downcast_ref::<VariableDagNode>() {
      self.name.cmp(&other.name())
    } else {
      Ordering::Less
    }
  }

  /*
    fn partial_compare_unstable(&self, partial_substitution: &mut Substitution, other: &dyn DagNode) -> Option<Ordering> {
      match partial_substitution.get(self.index) {
        None => {
          PartialOrdering::Unknown
        }
  
        Some(dag_node) => dag_node.borrow().compare(other).into(),
      }
    }
  */
  
  fn dagify_aux(&self) -> DagNodePtr {
    todo!()
  }

  fn analyse_constraint_propagation(&mut self, bound_uniquely: &mut NatSet) {
    bound_uniquely.insert(self.index as usize);
  }

  fn find_available_terms(&self, available_terms: &mut TermBag, eager_context: bool, at_top: bool) {
    if !at_top {
      available_terms.insert_matched_term(self.as_ptr(), eager_context);
    }
  }
}


impl Formattable for VariableTerm {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    let name = &self.name;
    let symbol: SymbolPtr = self.symbol();

    match style {
      FormatStyle::Default
      | FormatStyle::Simple
      | FormatStyle::Input => {
        // `X_Bool`
        
        match self.variable_type {
          VariableType::Blank        => write!(f, "{}_", name)?,
          VariableType::Sequence     => write!(f, "{}__", name)?,
          VariableType::NullSequence => write!(f, "{}___", name)?,
        }
        symbol.repr(f, FormatStyle::Default)
      }

      FormatStyle::Debug => {
        // `[variable<X><Bool><Blank>]`
        
        write!(f, "[{}<", name)?;
        symbol.repr(f, FormatStyle::Debug)?;
        
        match self.variable_type {
          VariableType::Blank        => write!(f, "><Blank>]"),
          VariableType::Sequence     => write!(f, "><Sequence>]"),
          VariableType::NullSequence => write!(f, "><NullSequence>]"),
        }
      }
    }

  }
}

impl_display_debug_for_formattable!(VariableTerm);
