use std::{
  cell::RefCell,
  cmp::max,
  collections::hash_map::Entry,
  rc::Rc,
};

use mod2_abs::{HashMap, IString, rc_cell, RcCell, heap_construct, SmallVec, smallvec};

use mod2_lib::{
  api::{
    variable_theory::VariableSymbol,
    built_in::{
      get_built_in_symbol,
      get_built_in_sort,
      Integer,
      StringBuiltIn,
      Float,
      NaturalNumber,
      Bool
    },
    Arity,
    symbol::{
      SymbolPtr,
      Symbol,
    },
    free_theory::FreeSymbol
  },
  core::{
    sort::{
      collection::SortCollection,
      SortPtr
    },
    symbol::{SymbolAttributes, SymbolType}
  },
};
use mod2_lib::core::symbol::{OpDeclaration, SymbolAttribute};
use crate::{
  parser::ast::{
    attribute::AttributeAST,
    BxSortIdAST,
    BxSortSpecAST
  }
};


pub(crate) type BxVariableDeclarationAST = Box<VariableDeclarationAST>;
pub(crate) type BxSymbolDeclarationAST = Box<SymbolDeclarationAST>;
pub type TypeSignature = SmallVec<[SortPtr; 1]>;

pub(crate) struct SymbolDeclarationAST {
  pub name      : IString,
  pub attributes: Vec<AttributeAST>,
  pub sort_spec : Option<BxSortSpecAST>, // Empty is the special "None" sort.
}

impl SymbolDeclarationAST {
  /// Creates a symbol for the symbol the AST describes and adds it to the symbol map.
  pub fn construct(
    &self,
    symbols: &mut HashMap<IString, SymbolPtr>,
    sorts  : &mut SortCollection
  ) {
    let maybe_type_signature: Option<TypeSignature> = match &self.sort_spec {
      Some(sort_spec) => {
        Some(sort_spec.construct(sorts))
      }
      None => unsafe {
        None
      }
    };

    construct_symbol_from_decl(
      symbols,
      sorts,
      self.name.clone(),
      maybe_type_signature,
      &self.attributes,
      SymbolType::Standard
    );
  }
}

pub(crate) struct VariableDeclarationAST {
  pub name      : IString,
  pub attributes: Vec<AttributeAST>,
  pub sort      : Option<BxSortIdAST>, // Empty is the special "Any" sort
}

impl VariableDeclarationAST {
  /// Creates a symbol for the variable the AST describes and adds it to the symbol map.
  pub fn construct(
    &self,
    symbols: &mut HashMap<IString, SymbolPtr>,
    sorts  : &mut SortCollection
  ) {
    let maybe_sort: Option<TypeSignature> = match &self.sort {
      Some(sort_id) => { 
        Some(smallvec![sort_id.construct(sorts)])
      },
      None => None
    };
    
    construct_symbol_from_decl(
      symbols,
      sorts,
      self.name.clone(),
      maybe_sort,
      &self.attributes,
      SymbolType::Variable
    );
  }
}

/// Constructs the symbol and inserts it into the provided `symbols` and `sorts` collections. This code is common between `VariableDeclarationAST` and `SymbolDeclarationAST`.
pub fn construct_symbol_from_decl(
  symbols       : &mut HashMap<IString, SymbolPtr>,
  sorts         : &mut SortCollection,
  name          : IString,
  type_signature: Option<TypeSignature>,
  attributes_ast: &Vec<AttributeAST>,
  symbol_type   : SymbolType,
)
{
  let attributes: SymbolAttributes = AttributeAST::construct_attributes(&attributes_ast);
  
  match symbols.entry(name.clone()) {

    Entry::Occupied(s) => {
      // ToDo: Under what circumstances would a symbol already exist? If the symbol is already declared, this
      //       should be a duplicate declaration and thus an error.
      panic!("duplicate variable declaration");
    },

    Entry::Vacant(entry) => {
      // The symbol doesn't exist. Create it.
      // Deduce the theory from the given attributes and instantiate the correct symbol type.
      let mut symbol = // the following match
        match symbol_type {
          SymbolType::Standard
          | SymbolType::Operator
          | SymbolType::Data => {
            // ToDo: Enrich this when more theories are implemented.
            let ptr = heap_construct!(FreeSymbol::new(name, Arity::Unspecified, attributes, symbol_type));
            SymbolPtr::new(ptr)
          }
  
  
          SymbolType::Variable => {
            let ptr = heap_construct!(VariableSymbol::new(name, Arity::Unspecified, attributes, symbol_type));
            SymbolPtr::new(ptr)
          }
  
          SymbolType::True => {
            unsafe{ get_built_in_symbol("true").unwrap_unchecked() }
          }
          SymbolType::False => {
            unsafe{ get_built_in_symbol("false").unwrap_unchecked() }
          }
          SymbolType::String => {
            unsafe{ get_built_in_symbol("String").unwrap_unchecked() }
          }
          SymbolType::Float => {
            unsafe{ get_built_in_symbol("Float").unwrap_unchecked() }
          }
          SymbolType::Integer => {
            unsafe{ get_built_in_symbol("Integer").unwrap_unchecked() }
          }
          SymbolType::NaturalNumber => {
            unsafe{ get_built_in_symbol("NaturalNumber").unwrap_unchecked() }
          }
        };

      if let Some(type_signature) = type_signature {
        let constructor_status = attributes.contains(SymbolAttribute::Constructor);
        let op_declaration = OpDeclaration::new(type_signature, constructor_status.into());
        symbol.add_op_declaration(op_declaration);
      }
      
      entry.insert(symbol);
    }

  };
}
