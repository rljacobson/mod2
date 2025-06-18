use std::{
  collections::hash_map::Entry,
  ops::Deref
};

use mod2_abs::{heap_construct, smallvec, HashMap, IString};
use mod2_lib::{
  api::{
    built_in::{
      BoolTerm,
      FloatTerm,
      IntegerTerm,
      NaturalTerm,
      StringTerm
    },
    free_theory::FreeSymbol,
    symbol::{
      Symbol,
      SymbolPtr,
    },
    term::{BxTerm, Term},
    variable_theory::{
      VariableSymbol,
      VariableTerm
    },
    Arity,
  },
  core::{
    sort::SortCollection,
    symbol::{
      OpDeclaration,
      SymbolAttribute,
      SymbolAttributes,
      SymbolType,
      TypeSignature
    }
  },
};
use crate::parser::ast::{
  attribute::AttributeAST,
  BxSortIdAST,
  BxSortSpecAST
};


pub(crate) type BxVariableDeclarationAST = Box<VariableDeclarationAST>;
pub(crate) type BxSymbolDeclarationAST   = Box<SymbolDeclarationAST>;

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
  ) -> BxTerm
  {
    let maybe_type_signature: Option<TypeSignature> = match &self.sort_spec {
      Some(sort_spec) => {
        Some(sort_spec.construct(sorts))
      }
      None => unsafe {
        None
      }
    };

    // ToDo: Right now we only make symbols of type `NASymbol<T>` and `FreeSymbol`. When other theories are implemented,
    //       the symbol type will need to be determined by the `attributes`.

    construct_symbol_term_from_decl(
      symbols,
      sorts,
      self.name.clone(),
      maybe_type_signature,
      &self.attributes,
      SymbolType::Standard
    )
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
  ) -> BxTerm
  {
    let maybe_sort: Option<TypeSignature> = match &self.sort {
      Some(sort_id) => {
        Some(smallvec![sort_id.construct(sorts)])
      },
      None => None
    };

    construct_symbol_term_from_decl(
      symbols,
      sorts,
      self.name.clone(),
      maybe_sort,
      &self.attributes,
      SymbolType::Variable
    )
  }
}

/// Constructs the symbol and inserts it into the provided `symbols` and `sorts` collections. This code is common between `VariableDeclarationAST` and `SymbolDeclarationAST`.
pub fn construct_symbol_term_from_decl(
  symbols       : &mut HashMap<IString, SymbolPtr>,
  sorts         : &mut SortCollection,
  name          : IString,
  type_signature: Option<TypeSignature>,
  attributes_ast: &Vec<AttributeAST>,
  symbol_type   : SymbolType,
) -> BxTerm
{
  let attributes: SymbolAttributes = AttributeAST::construct_attributes(&attributes_ast);

  // Variables have symbols named after their sort.
  let symbol_name =  match symbol_type {
    SymbolType::Variable => {
      match &type_signature {
        None => {
          IString::from("Any")
        }
        Some(signature) => {
          signature.last().unwrap().name.clone()
        }
      }
    },
    _ => name.clone()
  };

  let symbol = match symbols.entry(symbol_name.clone()) {

      Entry::Occupied(mut entry) => {
        // Operators (functions) can be overloaded. E.g., for A < B and X < Y, we could have
        //    symbol f: A A -> X;
        //    symbol f: B B -> Y;
        let symbol = entry.get_mut();

        if let Some(type_signature) = type_signature {
          let constructor_status = attributes.contains(SymbolAttribute::Constructor);
          let op_declaration     = OpDeclaration::new(type_signature, constructor_status.into());
          symbol.add_op_declaration(op_declaration);
        }

        *symbol
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
              assert!(sorts.len() > 0);
              let arity = Arity::new_unchecked((sorts.len()-1) as u16);
              let ptr = heap_construct!(FreeSymbol::new(name.clone(), arity, attributes, symbol_type));
              let mut symbol = SymbolPtr::new(ptr);

              if let Some(type_signature) = type_signature {
                let constructor_status = attributes.contains(SymbolAttribute::Constructor);
                let op_declaration = OpDeclaration::new(type_signature, constructor_status.into());
                symbol.add_op_declaration(op_declaration);
              }

              entry.insert(symbol);
              symbol
            }

            SymbolType::Variable => {
              // Variables have symbols named after their sort, so we use `symbol_name`.
              let ptr = heap_construct!(VariableSymbol::new(symbol_name, Arity::ZERO, attributes, symbol_type));
              let mut symbol = SymbolPtr::new(ptr);

              if let Some(type_signature) = type_signature {
                let constructor_status = attributes.contains(SymbolAttribute::Constructor);
                let op_declaration = OpDeclaration::new(type_signature, constructor_status.into());
                symbol.add_op_declaration(op_declaration);
              }

              entry.insert(symbol);
              symbol
            }

            // The following symbols do NOT get added to symbols owned by the module.
            // ToDo: Do some more validation, e.g. are the attributes compatible?
            SymbolType::True => {
              return Box::new(BoolTerm::from_str("true"));
            }
            SymbolType::False => {
              return Box::new(BoolTerm::from_str("false"));
            }
            SymbolType::String => {
              return Box::new(StringTerm::from_str(name.deref()));
            }
            SymbolType::Float => {
              return Box::new(FloatTerm::from_str(name.deref()));
            }
            SymbolType::Integer => {
              return Box::new(IntegerTerm::from_str(name.deref()));
            }
            SymbolType::NaturalNumber => {
              return Box::new(NaturalTerm::from_str(name.deref()));
            }
          };



        symbol
      }

    };

  match symbol_type {
    SymbolType::Variable => {
      Box::new(VariableTerm::new(name, symbol))
    }

    _ => {
      symbol.make_term(vec![])
    }
  }

}
