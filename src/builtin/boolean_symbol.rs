/*!

An integer literal is represented by a special integer literal symbol.

*/

use crate::{
  abstractions::IString,
  theory::{
    symbol::{
      Symbol,
      TheorySymbol,
      UNSPECIFIED
    },
    symbol_type::{CoreSymbolType, SymbolType},
  }
};

pub struct BooleanSymbol {
  value: bool, // ToDo: Maude uses a rope data structure.
}

impl BooleanSymbol {
  pub fn new(bool_literal: bool) -> Symbol {
    // ToDo: Should there be a core symbol type for each bool value?
    let core_type = match bool_literal {
      true  => CoreSymbolType::SystemTrue,
      false => CoreSymbolType::SystemFalse,
    };
    let symbol_type = SymbolType{
      core_type,
      attributes: Default::default(),
    };

    Symbol {
      // ToDo: What should the name be?
      name         : IString::from(""),
      arity        : UNSPECIFIED,
      order_hash   : Symbol::new_order_hash(0),
      symbol_type,
      sort_spec    : None,
      theory_symbol: Some(Box::new(
        BooleanSymbol{
          value: bool_literal
        }
      )),
    }
  }
}

impl TheorySymbol for BooleanSymbol {

}
