/*!

An integer literal is represented by a special integer literal symbol.

*/

use crate::{
  abstractions::{
    IString,
    NaturalNumber
  },
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

pub struct IntegerSymbol {
  value: NaturalNumber, // ToDo: Maude uses a rope data structure.
}

impl IntegerSymbol {
  pub fn new(integer_literal: NaturalNumber) -> Symbol {
    let symbol_type = SymbolType{
      core_type : CoreSymbolType::NaturalNumber,
      attributes: Default::default(),
    };

    Symbol {
      // ToDo: What should the name be? So far we have assumed the symbol name uniquely identifies the symbol. However,
      //       literals have no name.
      name         : IString::from(format!("{}", integer_literal).as_str()),
      arity        : UNSPECIFIED,
      symbol_type,
      sort_spec    : None,
      theory_symbol: Some(Box::new(
        IntegerSymbol{
          value: integer_literal
        }
      )),
    }
  }
}

impl TheorySymbol for IntegerSymbol {

}
