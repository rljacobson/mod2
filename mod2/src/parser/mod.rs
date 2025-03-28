/*!

Parser and AST. A "Global" `Module` is constructed from the AST. An intermediate AST representation is necessary for
checking uniqueness, types, etc.

*/

pub(crate) mod ast;
pub(crate) mod parser;



#[cfg(test)]
pub mod tests {
  use std::assert_matches::assert_matches;
  use lalrpop_util::lexer::Token;
  use lalrpop_util::ParseError;
  use crate::parser::ast::ModuleAST;
  use crate::parser::parser::ModuleParser;
  use super::*;

  pub fn parse_ex1() -> Result<Box<ModuleAST>, ()>{
    let path = "examples/example1.mod2";
    let text = match std::fs::read_to_string(path) {
      Ok(s) => { s }
      Err(e) => {
        panic!("Failed to read {}: {}", path, e);
      }
    };

    let parser = ModuleParser::new();
    let result: Result<Box<ModuleAST>, ParseError<usize, Token, &str>> =  parser.parse(&text);
    match result {
      Ok(ast) => {
        println!("SUCCESS!");
        Ok(ast)
      },
      Err(e) => {
        eprintln!("Parse error: {}", e);
        Err(())
      }
    }
  }


  #[test]
  fn test_ex1() {
    let result: Result<Box<ModuleAST>, ()> =  parse_ex1();
    assert_matches!(result, Ok(_));
  }
  
}
