/*!

Parser and AST. A "Global" `Module` is constructed from the AST. An intermediate AST representation is necessary for
checking uniqueness, types, etc.

*/


use crate::add;

mod ast;
mod parser;



#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ex1() {
    let path = "examples/example1.mod2";
    let text = match std::fs::read_to_string(path) {
      Ok(s) => { s }
      Err(e) => {
        panic!("Failed to read {}: {}", path, e);
      }
    };

    let parser = parser::ModuleParser::new();
    let result =  parser.parse(text.as_str());
    match result {
      Ok(_) => {
        println!("SUCCESS!");
      },
      Err(e) => {
        eprintln!("Parse error: {}", e);
      }
    }
  }
}
