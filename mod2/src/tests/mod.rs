/*!

These are tests for mod2-lib that are difficult to do without parsing and module construction.

*/

use std::collections::HashMap;
use lalrpop_util::{
  lexer::Token,
  ParseError
};
use mod2_abs::IString;
use mod2_lib::core::sort::collection::SortCollection;
use crate::parse_to_module;

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

  let module = parse_to_module(source);
  assert!(module.is_ok());
  let module = module.unwrap();

  println!("{:?}", module);

  let mut f      = module.symbols[&IString::from("f")];
  let f2         = f.clone();
  let sort_table = &mut f.core_mut().sort_table;
  
  sort_table.compile_op_declaration(f2);
  println!("Raw sort diagram: {:?}", sort_table.sort_diagram);
  {
    let mut out = String::new();
    sort_table.dump_sort_diagram(&mut out, 2).unwrap();
    println!("{}", out);
  }
}


#[test]
fn test_ex1_construction() {

  let path = "examples/example1.mod2";
  let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
    panic!("Failed to read {}: {}", path, e);
  });

  let constructed = parse_to_module(&*text).unwrap();
  println!("{:?}", constructed);
}

#[test]
fn test_parse_term() {
  let mut parser = crate::parser::parser::TermParser::default();
  let input = "f(\"hello\", 1, 2.0, true, g(x, y, z, false))";
  let term = parser.parse(input).unwrap();

  let mut symbols = HashMap::new();
  let mut sorts = SortCollection::new();
  let mut variables = HashMap::new();

  let term = term.construct(&mut symbols, &mut sorts, &mut variables);
  println!("{:?}", term);
}