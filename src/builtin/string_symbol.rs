/*!

A string literal is represented by a special string literal symbol.

*/

use crate::{
  abstractions::IString,
  theory::{
    symbol::{
      Symbol,
      TheorySymbol,
      UNSPECIFIED
    },
    symbol_type::{
      CoreSymbolType,
      SymbolType
    },
  }
};
use crate::core::sort::sort_spec::SortSpec;

pub struct StringSymbol {
  value: String, // ToDo: Maude uses a rope data structure.
}

impl StringSymbol {
  pub fn new(string_literal: String) -> Symbol {
    let symbol_type = SymbolType{
      core_type : CoreSymbolType::String,
      attributes: Default::default(),
    };

    Symbol {
      // ToDo: What should the name be? So far we have assumed the symbol name uniquely identifies the symbol. However,
      //       strings have no name.
      name         : IString::from(""),
      arity        : UNSPECIFIED,
      symbol_type,
      sort_spec    : Some(Box::new(SortSpec::Any)),
      theory_symbol: Some(Box::new(StringSymbol{value: string_literal})),
    }
  }
}

impl TheorySymbol for StringSymbol {

}
