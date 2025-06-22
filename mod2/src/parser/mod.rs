/*!

Parser and AST. A "Global" `Module` is constructed from the AST. An intermediate AST representation is necessary for
checking uniqueness, types, etc.

*/

use std::collections::HashMap;
use lalrpop_util::{
  lexer::Token,
  ParseError
};
use mod2_abs::IString;
use mod2_lib::{
  core::{
    BxModule,
    sort::SortCollection
  },
  api::{
    BxTerm,
    SymbolPtr
  },
};
use crate::parser::parser::{ModuleParser, TermParser};

pub(crate) mod ast;
pub(crate) mod parser;

/// Public API to create `mod2-lib::Term` instances from source.
pub fn parse_to_term<'input>(
  input: &'input str,
  symbols: &mut HashMap<IString, SymbolPtr>,
  sorts: &mut SortCollection,
  variables: &mut HashMap<IString, BxTerm>
) -> Result<BxTerm, ParseError<usize, Token<'input>, &'static str>>
{
  let mut parser = TermParser::default();
  let term_ast   = parser.parse(input)?;

  Ok(term_ast.construct(symbols, sorts, variables))
}

/// Public API to create a `mod2-lib::Module` instance from source.
pub fn parse_to_module<'input>(input: &'input str) -> Result<BxModule, ParseError<usize, Token<'input>, &'static str>>
{
  let mut parser = ModuleParser::default();
  let module_ast = parser.parse(input)?;

  Ok(module_ast.construct_module())
}




#[cfg(test)]
pub mod tests {
  use std::assert_matches::assert_matches;
  use lalrpop_util::{lexer::Token, ParseError};
  use crate::parser::{ast::ModuleAST, parser::ModuleParser};
  use super::*;

  #[test]
  pub fn text_parse_ex1() {
    let path = "examples/example1.mod2";
    let text = match std::fs::read_to_string(path) {
      Ok(s) => { s }
      Err(e) => {
        panic!("Failed to read {}: {}", path, e);
      }
    };
    let result: Result<BxModule, ParseError<usize, Token, &str>> =  parse_to_module(&text);
    
    match result {
      Ok(module) => {
        println!("SUCCESS!");
      }
      Err(e) => {
        panic!("Parse error: {}", e);
      }
    }
  }


  #[test]
  fn test_parse_term() {
    let input = "f(\"hello\", 1, 2.0, true, g(x, y, z, false))";

    let mut symbols   = HashMap::new();
    let mut sorts     = SortCollection::new();
    let mut variables = HashMap::new();
    let term          = parse_to_term(input, &mut symbols, &mut sorts, &mut variables).unwrap();

    println!("{:?}", term);
  }

}
