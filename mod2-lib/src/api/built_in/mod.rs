/*!
We subvert a lot of protections in order to do this. We justify it with the
fact that after construction the containers and their contents are never
moved nor mutated in any way, and they live for the life of the program.
*/

mod nonalgebraic_term;
mod nonalgebraic_symbol;

use std::{
  collections::HashMap,
  sync::Arc
};
use once_cell::sync::Lazy;
use mod2_abs::{smallvec, IString};

use crate::{
  api::{
    symbol::SymbolPtr,
    Arity
  },
  core::{
    sort::{
      Sort,
      SortPtr
    },
    symbol::{
      OpDeclaration,
      SymbolAttribute,
      SymbolCore,
      SymbolType
    }
  }
};

pub use nonalgebraic_term::*;
pub use nonalgebraic_symbol::*;

// Built-in Types
pub type Bool    = bool;
/// Floating Point Numbers
pub type Float   = f64;
/// Signed Integers
pub type Integer = i64;
/// Nonnegative Integers
pub type NaturalNumber = u64;
/// Strings
pub type StringBuiltIn = String;


macro_rules! make_symbol {
    ($sort_name:expr, $symbol_name:expr, $symbol_type:expr) => {
        {
          let sort            = get_built_in_sort($sort_name).unwrap();
          let symbol_name     = IString::from($symbol_name);
          let symbol_core = SymbolCore::new(
            symbol_name.clone(),
            Arity::Value(0),
            SymbolAttribute::Constructor.into(),
            $symbol_type,
          );
          let symbol          = Box::new(BoolSymbol::new(symbol_core));
          let mut symbol_ptr  = SymbolPtr::new(Box::into_raw(symbol));
          let op_declaration  = OpDeclaration::new(smallvec![sort], true.into());
          let symbol_ptr_copy = symbol_ptr; // Force copy
          symbol_ptr.add_op_declaration(symbol_ptr_copy, op_declaration);
          
          ($symbol_name, symbol_ptr)
        }
    };
}

static BUILT_IN_SORTS: Lazy<HashMap<&'static str, Sort>> = Lazy::new(|| {
  let mut sorts = HashMap::default();
  // ToDo: Warn when a user shadows a built-in.
  {
    let name = "Float";
    let sort = Sort::new(IString::from(name));
    sorts.insert(name, sort);
  }
  {
    let name = "Integer";
    let sort = Sort::new(IString::from(name));
    sorts.insert(name, sort);
  }
  {
    let name = "NaturalNumber";
    let sort = Sort::new(IString::from(name));
    sorts.insert(name, sort);
  }
  {
    let name = "String";
    let sort = Sort::new(IString::from(name));
    sorts.insert(name, sort);
  }
  {
    let name = "Bool";
    let sort = Sort::new(IString::from(name));
    sorts.insert(name, sort);
  }
  {
    let name = "Any";
    let sort = Sort::new(IString::from(name));
    sorts.insert(name, sort);
  }
  {
    let name = "None";
    let sort = Sort::new(IString::from(name));
    sorts.insert(name, sort);
  }
  
  println!("Initialized built-in sorts: {:?}", sorts);
  
  sorts
});

static BUILT_IN_SYMBOLS: Lazy<HashMap<&'static str, SymbolPtr>> = Lazy::new(|| {
  let mut symbols = HashMap::default();
  // ToDo: Warn when a user shadows a built-in.
  // Bool true
  {
    let (symbol_name, symbol_ptr) = make_symbol!("Bool", "true", SymbolType::True);
    symbols.insert(symbol_name, symbol_ptr);
  }
  // Bool false
  {
    let (symbol_name, symbol_ptr) = make_symbol!("Bool", "false", SymbolType::False);
    symbols.insert(symbol_name, symbol_ptr);
  }
  // String
  {
    let (symbol_name, symbol_ptr) = make_symbol!("String", "String", SymbolType::String);
    symbols.insert(symbol_name, symbol_ptr);
  }
  // Float
  {
    let (symbol_name, symbol_ptr) = make_symbol!("Float", "Float", SymbolType::Float);
    symbols.insert(symbol_name, symbol_ptr);
  }
  // Integer
  {
    let (symbol_name, symbol_ptr) = make_symbol!("Integer", "Integer", SymbolType::Integer);
    symbols.insert(symbol_name, symbol_ptr);
  }
  // NaturalNumber
  {
    let (symbol_name, symbol_ptr) = make_symbol!("NaturalNumber", "NaturalNumber", SymbolType::NaturalNumber);
    symbols.insert(symbol_name, symbol_ptr);
  }
  
  symbols
});

pub fn get_built_in_sort(name: &str) -> Option<SortPtr> {
  let sort: &Sort = BUILT_IN_SORTS.get(name)?;
  Some(SortPtr::new((sort as *const Sort) as *mut Sort))
}

pub fn get_built_in_symbol(name: &str) -> Option<SymbolPtr> {
  let symbol: SymbolPtr = *BUILT_IN_SYMBOLS.get(name)?;
  Some(symbol)
}


#[cfg(test)]
mod tests {
  use std::ops::Deref;
  use super::*;
  
  #[test]
  fn test_built_in_sorts() {
    let maybe_bool_sort = get_built_in_sort("Bool");
    assert!(maybe_bool_sort.is_some());
    
    let bool_sort = maybe_bool_sort.unwrap();
    assert_eq!(bool_sort.name.deref(), "Bool")
  }

  #[test]
  fn test_built_in_symbols() {
    let maybe_true_symbol = get_built_in_symbol("true");
    assert!(maybe_true_symbol.is_some());

    let true_symbol = maybe_true_symbol.unwrap();
    assert_eq!(true_symbol.name().deref(), "true")
  }
}