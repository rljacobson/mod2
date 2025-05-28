use std::sync::{LazyLock, Mutex};
use rand::Rng;

use mod2_abs::{heap_construct, IString};

use crate::{
  api::{
    dag_node::{
      DagNode,
      DagNodePtr
    },
    free_theory::{
      FreeDagNode,
      FreeSymbol
    },
    symbol::{
      Symbol,
      SymbolPtr
    },
    variable_theory::{
      VariableDagNode,
      VariableSymbol
    },
    Arity
  },
  core::{
    dag_node_core::{
      DagNodeCore,
      ThinDagNodePtr
    },
    gc::{
      allocate_dag_node,
      node_allocator::acquire_node_allocator,
      root_container::RootVec
    },
    EquationalTheory
  }
};


// ToDo: Figure out why multithreading breaks the tests.
// Force GC tests to run serially for consistent behavior.
static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

/*
Recursively builds a random tree of `DagNode`s with a given height and arity rules.

Because this function holds on to iterators of `NodeVec`s, the GC cannot run during
the building of the tree. Run the GC before or after.

 - `symbols`: List of `Symbol` objects of each arity from 0 to `max_width`.
 - `parent`: Pointer to the current parent node.
 - `max_height`: Maximum allowed height for the tree.
*/
pub fn build_random_tree(
  symbols   : &mut [SymbolPtr],
  parent    : DagNodePtr,
  max_height: usize,
  max_width : usize,
  min_width : usize,
) {
  if max_height == 0 {
    return; // Reached the maximum depth
  }

  // idiot-proof
  let min_width = std::cmp::min(max_width, min_width);
  let max_width = std::cmp::max(max_width, min_width);

  let mut rng   = rand::rng();

  // Get the parent node's arity from its symbol
  let parent_arity = if let Arity::Value(v) = parent.arity() { v as usize } else { 0 };

  // For each child based on the parent's arity, create a new node
  for i in 0..parent_arity as usize {
    // Determine the arity of the child node
    let child_arity = if max_height == 1 {
      0 // Leaf nodes must have arity 0
    } else {
      rng.random_range(min_width..=max_width) // Random arity between min_width and max_width
    };

    // Create the child node with the symbol corresponding to its arity
    let child_symbol: SymbolPtr = symbols[child_arity];
    let child_node   = FreeDagNode::new(child_symbol);

    // Insert the child into the parent node
    let mut parent_mut = parent;
    if let Arity::Value(v) = parent_mut.arity(){
      if i > v as usize {
        panic!("Incorrect arity");
      }
    }
    parent_mut.insert_child(child_node);

    // Recursively build the subtree for the child
    build_random_tree(symbols, child_node, max_height - 1, max_width, min_width);
  }
}

/// Recursively prints a tree structure using ASCII box-drawing symbols.
///
/// - `node`: The current node to print.
/// - `prefix`: The string prefix to apply to the current node's line.
/// - `is_tail`: Whether the current node is the last child of its parent.
pub fn print_tree(node: DagNodePtr, prefix: String, is_tail: bool) {
  let is_head = prefix.is_empty();
  let arity = if let Arity::Value(v) = node.arity() {
    v
  } else {
    0
  };

  if arity as usize != node.len() {
    panic!("Incorrect arity/len. arity: {}  len: {}", arity, node.len());
  }

  // Print the current node
  let new_prefix = if is_head {
    ""
  } else {
    if is_tail { "╰──" } else { "├──" }
  };
  println!(
    "{}{}{}",
    prefix,
    new_prefix,
    node
  );

  // Determine the new prefix for children
  let new_prefix = if is_tail {
    format!("{}    ", prefix)
  } else if is_head {
    format!(" ")
  }
  else {
    format!("{}│   ", prefix)
  };

  // Print each child
  for (i, child_ptr) in node.iter_args().enumerate() {
    print_tree(
      child_ptr,
      new_prefix.clone(),
      i == node.len() - 1, // Is this the last child?
    );
  }
}

fn make_symbols() -> Vec<SymbolPtr> {
  (0..=10).map(|x| {
            // let name = IString::from(format!("sym({})", x).as_str());
            let name = IString::from("sym");
            SymbolPtr::new(heap_construct!(FreeSymbol::with_arity(name, Arity::Value(x))))
          })
      .collect::<Vec<_>>()
}

#[test]
fn test_allocate_dag_node() {
  let _guard = TEST_MUTEX.lock();
  let node_ptr: ThinDagNodePtr   = allocate_dag_node();
  let node_mut: &mut DagNodeCore = match unsafe { node_ptr.as_mut() } {
    None => {
      panic!("allocate_dag_node returned None");
    }
    Some(node) => { node }
  };

  node_mut.theory_tag = EquationalTheory::Free;
}


#[test]
fn test_dag_creation() {
  let _guard = TEST_MUTEX.lock();
  let mut symbols = make_symbols();

  let root = FreeDagNode::new(symbols[3]);
  let _root_container = RootVec::with_node(root);

  // Maximum tree height
  let max_height: usize = 6;
  let max_width : usize = 3;

  // Recursively build the random tree
  build_random_tree(&mut symbols, root, max_height, max_width, 0);
  print_tree(root, String::new(), false);
  // println!("Symbols: {:?}", symbols);
  #[cfg(feature = "gc_debug")]
  acquire_node_allocator("dump_memory_variables").dump_memory_variables()
}


#[test]
fn test_garbage_collection() {
  let _guard = TEST_MUTEX.lock();
  let mut symbols = make_symbols();

  for _ in 0..100 {
    let mut root_vec = Vec::with_capacity(10);

    for _ in 0..10 {
      let root: DagNodePtr = DagNodeCore::new(symbols[4]);
      let root_container = RootVec::with_node(root);
      root_vec.push(root_container);

      // Maximum tree height
      let max_height: usize = 6; // exponent
      let max_width : usize = 4; // base

      // Recursively build the random tree
      build_random_tree(&mut symbols, root, max_height, max_width, 0);
    }
    acquire_node_allocator("ok_to_collect_garbage").ok_to_collect_garbage();

    // root_vec dropped
  }
  #[cfg(feature = "gc_debug")]
  acquire_node_allocator("dump_memory_variables").dump_memory_variables()
}


#[test]
fn test_arena_exhaustion() {
  let _guard = TEST_MUTEX.lock();
  let symbol = FreeSymbol::with_arity(IString::from("mysymbol"), Arity::Value(1));
  let symbol_ptr = SymbolPtr::new(heap_construct!(symbol));
  let root: DagNodePtr = DagNodeCore::new(symbol_ptr);
  println!("root: {}", root);

  let _root_container = RootVec::with_node(root);

  let mut last_node = root;

  for _ in 1..=10000 {
    let node_ptr = allocate_dag_node();
    let node_mut = match unsafe { node_ptr.as_mut() } {
      None => {
        panic!("allocate_dag_node returned None");
      }
      Some(node) => {
        node
      }
    };
    node_mut.theory_tag = EquationalTheory::Free;
    let node_ptr = DagNodeCore::upgrade(node_ptr);
    last_node.insert_child(node_ptr);
    last_node    = node_ptr;
  }

}


#[test]
fn create_destroy_variable_dag_node() {
  let name = IString::from("Hello");
  {
    let symbol = VariableSymbol::with_name(name.clone());
    {
      let node = VariableDagNode::new(symbol.as_ptr(), name.clone(), 0);
      println!("{}", node);
      // Node goes out of scope. 
    }
    {
      let mut allocator = acquire_node_allocator("VariableDagNode finalizer test");
      allocator.collect_garbage();
    }
    // Create a dummy `DagNode` to force lazy sweep
    let dummy_symbol = FreeSymbol::with_arity("Free".into(), Arity::None);
    let dummy_node = FreeDagNode::new(dummy_symbol.as_ptr());
    _ = dummy_node.len(); // Don't optimize away

    println!("{}", symbol);
  }
  println!("{}", name);
}