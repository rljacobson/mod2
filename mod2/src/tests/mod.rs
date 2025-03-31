/*!

These are tests for mod2-lib that are difficult to do without parsing and module construction.

*/

use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError;
use mod2_abs::IString;
use crate::parser::ast::ModuleAST;
use crate::parser::parser::ModuleParser;
use crate::parser::tests::parse_ex1;

fn parse_string(text: &str) -> Result<Box<ModuleAST>, ()> {
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
fn test_sort_table_sort_diagram(){
  let source = r"
  sort A < B;
  sort B < C;

  sort X < Y;
  sort Y < Z;

  symbol f: A A -> X;
  symbol f: B B -> Y;
  symbol f: C C -> Z;

  // We would compute the sort of `f(p, q)`, `f(r, r)`, etc.
  symbol p: A;
  symbol q: B;
  symbol r: C;
  ";

  let ast = parse_string(source);
  assert!(ast.is_ok());
  let module = ast.unwrap().construct_module();

  println!("{:?}", module);

  let mut f      = module.symbols[&IString::from("f")];
  let sort_table = f.core_mut().sort_table.as_mut().unwrap();
  
  sort_table.compile_op_declaration();
  println!("Raw sort diagram: {:?}", sort_table.sort_diagram);
  {
    let mut out = String::new();
    sort_table.dump_sort_diagram(&mut out, 2).unwrap();
    println!("{}", out);
  }
}


#[test]
fn test_ex1_construction() {
  let ast: Box<ModuleAST> =  parse_ex1().expect("Failed to parse module");
  let constructed = ast.construct_module();
  println!("{:?}", constructed);
}
