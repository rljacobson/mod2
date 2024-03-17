/*

We generate the LALRPOP parser in the source tree so that our IDE can index it. RustRover (and
JetBrains' Rust support generally) doesn't understand LALRPOP specs or how to index the generated
code.

*/

fn main() {
  // Uncomment if generating parser out of source tree.
  // lalrpop::process_root().unwrap();

  // Comment out if generating parser out of source tree.
  lalrpop::Configuration::new()
      .generate_in_source_tree()
      .process().unwrap();
}
