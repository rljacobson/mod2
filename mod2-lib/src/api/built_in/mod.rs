/*!
We subvert a lot of protections in order to do this. We justify it with the
fact that after construction the containers and their contents are never
moved nor mutated in any way, and they live for the life of the program.
*/

use std::{
  collections::HashMap,
  sync::Arc
};

use once_cell::sync::Lazy;
use mod2_abs::IString;

use crate::{
  api::{
    Arity,
    symbol::{
      Symbol, 
      SymbolAttribute, 
      SymbolAttributes, 
      SymbolPtr, 
      SymbolType
    }
  },
  core::sort::{
    sort_spec::SortSpec, 
    Sort, 
    SortPtr
  },
};

static BUILT_IN_SORTS: Lazy<Arc<HashMap<IString, Sort>>> = Lazy::new(|| {
  let mut sorts = HashMap::new();
  // ToDo: Warn when a user shadows a built-in.
  
  let name = IString::from("Bool");
  let sort = Sort::new(name.clone());
  sorts.insert(name, sort);

  Arc::new(sorts)
});

static BUILT_IN_SYMBOLS: Lazy<Arc<HashMap<IString, Symbol>>> = Lazy::new(|| {
  let mut symbols = HashMap::new();
  // ToDo: Warn when a user shadows a built-in.
  
  let sort_name = IString::from("Bool");
  let sort = get_built_in_sort(&sort_name).unwrap();
  let name: IString = IString::from("true");
  let symbol = Symbol::new(
    name.clone(),
    Arity::Value(0),
    SymbolAttribute::Constructor.into(),
    SymbolType::True,
    SortSpec::Sort(sort),
  );
  symbols.insert(name, symbol);

  let name: IString = IString::from("false");
  let symbol = Symbol::new(
    name.clone(),
    Arity::Value(0),
    SymbolAttribute::Constructor.into(),
    SymbolType::False,
    SortSpec::Sort(sort),
  );
  symbols.insert(name, symbol);
  

  Arc::new(symbols)
});

pub fn get_built_in_sort(name: &str) -> Option<SortPtr> {
  let name = IString::from(name);
  let sort: &Sort = BUILT_IN_SORTS.get(&name)?;
  Some((sort as *const Sort) as *mut Sort)
}

pub fn get_built_in_symbol(name: &str) -> Option<SymbolPtr> {
  let name = IString::from(name);
  let symbol: &Symbol = BUILT_IN_SYMBOLS.get(&name)?;
  Some((symbol as *const Symbol) as *mut Symbol)
}