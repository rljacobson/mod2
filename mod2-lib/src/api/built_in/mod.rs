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
      SymbolAttribute,
      SymbolCore,
      SymbolType
    }
  }
};

pub use nonalgebraic_term::*;
pub use nonalgebraic_symbol::*;
use crate::core::symbol::OpDeclaration;

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


static BUILT_IN_SORTS: Lazy<Arc<HashMap<IString, Sort>>> = Lazy::new(|| {
  let mut sorts = HashMap::new();
  // ToDo: Warn when a user shadows a built-in.
  {
    let name = IString::from("Float");
    let sort = Sort::new(name.clone());
    sorts.insert(name, sort);
  }
  {
    let name = IString::from("Integer");
    let sort = Sort::new(name.clone());
    sorts.insert(name, sort);
  }
  {
    let name = IString::from("NaturalNumber");
    let sort = Sort::new(name.clone());
    sorts.insert(name, sort);
  }
  {
    let name = IString::from("String");
    let sort = Sort::new(name.clone());
    sorts.insert(name, sort);
  }
  {
    let name = IString::from("Bool");
    let sort = Sort::new(name.clone());
    sorts.insert(name, sort);
  }
  {
    let name = IString::from("Any");
    let sort = Sort::new(name.clone());
    sorts.insert(name, sort);
  }
  {
    let name = IString::from("None");
    let sort = Sort::new(name.clone());
    sorts.insert(name, sort);
  }
  Arc::new(sorts)
});

macro_rules! make_symbol {
    ($sort_name:expr, $symbol_name:expr, $symbol_type:expr) => {
        {
          let sort_name   = IString::from($sort_name);
          let sort        = get_built_in_sort(&sort_name).unwrap();
          let symbol_name = IString::from($symbol_name);
          let mut symbol_core = SymbolCore::new(
            symbol_name.clone(),
            Arity::Value(0),
            SymbolAttribute::Constructor.into(),
            $symbol_type,
            // sort,
          );
          let op_declaration = OpDeclaration::new(smallvec![sort], true.into());
          symbol_core.add_op_declaration(op_declaration);
          
          let symbol     = Box::new(BoolSymbol::new(symbol_core));
          let symbol_ptr = SymbolPtr::new(Box::into_raw(symbol));
          (symbol_name, symbol_ptr)
        }
    };
}

static BUILT_IN_SYMBOLS: Lazy<Arc<HashMap<IString, SymbolPtr>>> = Lazy::new(|| {
  let mut symbols = HashMap::new();
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

  Arc::new(symbols)
});

pub fn get_built_in_sort(name: &str) -> Option<SortPtr> {
  let name = IString::from(name);
  let sort: &Sort = BUILT_IN_SORTS.get(&name)?;
  
  Some(SortPtr::new((sort as *const Sort) as *mut Sort))
}

pub fn get_built_in_symbol(name: &str) -> Option<SymbolPtr> {
  let name = IString::from(name);
  let symbol: SymbolPtr = *BUILT_IN_SYMBOLS.get(&name)?;
  Some(symbol)
}
