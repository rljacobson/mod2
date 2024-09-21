/*!

Parser and AST. A "Global" `Module` is constructed from the AST. An intermediate AST representation is necessary for
checking uniqueness, types, etc.

*/


use crate::add;

pub(crate) mod ast;
pub(crate) mod parser;



#[cfg(test)]
mod tests {
  use std::assert_matches::assert_matches;
  use lalrpop_util::lexer::Token;
  use lalrpop_util::ParseError;
  use crate::parser::ast::ModuleAST;
  use super::*;

  fn parse_ex1() -> Result<Box<ModuleAST>, ()>{
    let path = "examples/example1.mod2";
    let text = match std::fs::read_to_string(path) {
      Ok(s) => { s }
      Err(e) => {
        panic!("Failed to read {}: {}", path, e);
      }
    };

    let parser = parser::ModuleParser::new();
    let result: Result<Box<ModuleAST>, ParseError<usize, Token, &str>> =  parser.parse(&text);
    match result {
      Ok(ast) => {
        println!("SUCCESS!");
        return Ok(ast);
      },
      Err(e) => {
        eprintln!("Parse error: {}", e);
        return Err(());
      }
    }
  }

  #[test]
  fn test_ex1() {
    let result: Result<Box<ModuleAST>, ()> =  parse_ex1();
    assert_matches!(result, Ok(_));
  }

  #[test]
  fn test_ex1_construction() {
    let ast: Box<ModuleAST> =  parse_ex1().expect("Failed to parse module");
    let constructed = ast.construct_module();
  }
}
