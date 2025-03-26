/*!

Attributes can appear in brackets at the end of symbol, variable, and operator declarations
and primarily affect the construction of `SymbolType`. (See the `theory::symbol_type` module).

```ebnf
Attribute :=
    "assoc" | "associative"
    | "comm" | "commutative"
    | "ctor" | "constructor"
    | "id" "(" Term ")"
    ;

AttributeSpec := "[" AttributeList "]" ;
```

*/

use crate::{
  parser::ast::BxTermAST
};

use mod2_lib::{
  core::symbol::{
    SymbolAttribute,
    SymbolAttributes
  }
};

/// Differs from the non-AST `TheoryAttribute` in that `TheoryAttributeAST::Identity(BxPatternAST)` holds a
/// `BxPatternAST`.
pub(crate) enum AttributeAST {
  Associative,
  Commutative,
  Constructor,
  Identity(BxTermAST)
  // ToDo: Figure out how to keep track of the identity element. (How does Maude do this?)
}

impl AttributeAST {
  /// Gives the `SymbolAttributes` value equivalent of `self`. Used in the `AttributeAST::construct_attributes`
  /// static method.
  fn to_attributes(&self) -> SymbolAttributes {
    match self {
      AttributeAST::Associative => SymbolAttribute::Associative.into(),
      AttributeAST::Commutative => SymbolAttribute::Commutative.into(),
      AttributeAST::Constructor => SymbolAttribute::Constructor.into(),
      AttributeAST::Identity(_) => SymbolAttribute::LeftIdentity | SymbolAttribute::RightIdentity,
    }
  }

  /// Converts a vector of `TheoryAttributeAST` values into a `TheoryAttributes` (one byte bitflag).
  pub fn construct_attributes(attribute_ast: &Vec<AttributeAST>) -> SymbolAttributes {
    let mut attributes = SymbolAttributes::empty();
    for attribute in attribute_ast {
      attributes |= attribute.to_attributes();
    }
    attributes
  }
}
