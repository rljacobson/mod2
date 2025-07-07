/*!
A discrimination net for the Free theory.

A discrimination net is a data structure used to optimize rule-based pattern matching. It improves
efficiency by organizing the conditions of the rules in a tree-like structure, enabling the system
to check many rules simultaneously. An input data item is classified by conducting a series of
individual predicate tests. The internal nodes of the tree are the conditions that are tested, while
the end nodes of the net symbolize the outcomes for the different possible predicate sequences.

*/

use mod2_abs::HashSet;

use crate::{
  api::{
    free_theory::{
      remainder::{
        FreeRemainder,
        FreeTerm,
        FreeRemainderPtr,
        Speed
      },
      FreeSymbol
    },
    DagNodePtr,
    SymbolPtr
  },
  core::{
    ArgIndex,
    DagNodeArguments,
    SlotIndex,
    SymbolIndex,
    pre_equation::{BxPreEquation, PreEquation},
    rewriting_context::RewritingContext,
  }
};


pub type PatternSet = HashSet<SlotIndex>;
pub type BxFreeNet  = Box<FreeNet>;
// pub type SharedNodeList = SharedVector<DagNodePtr>;

struct Triple {
  symbol : SymbolPtr,
  slot   : SlotIndex,
  subtree: i32,
}

// region Ordering of Triples
impl Eq for Triple {}

impl PartialEq for Triple {
  fn eq(&self, other: &Self) -> bool {
    self.symbol.index_within_parent_module() == other.symbol.index_within_parent_module()
  }
}

impl PartialOrd for Triple {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for Triple {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.symbol
        .core()
        .index_within_parent_module
        .cmp(&other.symbol.index_within_parent_module())
  }
}
// endregion


#[derive(Copy, Clone, Default)]
struct NotEqual {
  pub greater: i32,
  pub less   : i32,
}

#[derive(Copy, Clone, Default)]
struct TestNode {
  /// Index of next test node to take for > and < cases (-ve encodes index of applicable list, 0 encodes failure)
  not_equal:    NotEqual,
  /// Stack slot to get free DagNode argument list from (-1 indicates use old argument)
  position    : SlotIndex,
  /// Index of argument to test
  arg_index   : ArgIndex,
  /// Index within module of symbol we test against
  symbol_index: SymbolIndex,
  /// Index of stack slot to store free dagnode argument list in (-1 indicates do not store)
  slot        : SlotIndex,
  /// Index of next test node to take for == case (-ve encode index of applicable list)
  equal       : i32,
}

/// This is a self-referential data structure that cannot be moved after `FreeNet::build_remainders()`
/// is called.
#[derive(Default)]
pub struct FreeNet {
  stack          : Vec<DagNodeArguments>,
  net            : Vec<TestNode>,
  /// Holds pointers to the `FreeRemainder`s in `self.remainders` that are valid for the lifetime of `self`
  /// as long as `self` is not moved.
  fast_applicable: Vec<Vec<FreeRemainderPtr>>,
  remainders     : Vec<Option<FreeRemainder>>,
  applicable     : Vec<PatternSet>,
  fast           : bool,
}

impl FreeNet {
  pub fn new() -> Self {
    FreeNet {
      fast: true,
      ..FreeNet::default()
    }
  }

  pub fn fast_handling(&self) -> bool {
    self.fast
  }

  fn allocate_node(&mut self, nr_match_arcs: usize) -> usize {
    let len = self.net.len();
    self.net.resize(len + nr_match_arcs, TestNode::default());
    len
  }

  fn fill_out_node(
    &mut self,
    mut node_index: i32,
    position      : SlotIndex,
    arg_index     : ArgIndex,
    symbols       : &Vec<SymbolPtr>,
    targets       : &Vec<i32>,
    slots         : &Vec<SlotIndex>,
    neq_target    : i32,
  ) {
    let symbol_count = symbols.len();
    let mut triples  = Vec::with_capacity(symbol_count);

    for i in 0..symbol_count {
      triples.push(Triple {
        symbol : symbols[i].clone(),
        slot   : slots[i],
        subtree: targets[i],
      });
    }

    triples.sort();
    self.build_ternary_tree(
      &mut node_index,
      &mut triples,
      0,
      symbol_count - 1,
      neq_target,
      position,
      arg_index,
    );
  }

  fn add_remainder_list(&mut self, live_set: PatternSet) -> i32 {
    let index = self.applicable.len();
    self.applicable.push(live_set);
    // The bitwise NOT indicates this is an index into `applicable`.
    !(index as i32)
  }

  fn translate_slots(&mut self, real_slot_count: usize, slot_translation: &Vec<SlotIndex>) {
    self.stack.reserve(real_slot_count);

    for node in &mut self.net {
      node.slot = if node.slot.is_index(){
        slot_translation[node.slot.idx()]
      } else {
        SlotIndex::None
      };
      node.position = if node.position.is_index() {
        slot_translation[node.position.idx()]
      } else {
        SlotIndex::None
      };
    }
  }

  fn build_remainders(
    &mut self,
    equations       : &mut Vec<PreEquation>,
    patterns_used   : &PatternSet,
    slot_translation: &Vec<SlotIndex>,
  ) {
    let equation_count = equations.len();
    self.remainders.resize_with(equation_count, | | None);

    for i in patterns_used {
      let mut equation = equations[i.idx()].as_ptr();
      // ToDo: Get rid of this:
      let equation2 = equations[i.idx()].as_ptr();

      if let Some(free_term) = equation.lhs_term
                                       .as_any_mut()
                                       .downcast_mut::<FreeTerm>()
      {
        let remainder = free_term.compile_remainder(equation2, slot_translation);
        // If a remainder doesn't have fast handling, neither can the discrimination net.
        self.fast = remainder.fast != Speed::Slow;
        self.remainders[i.idx()] = Some(remainder);
      } else {
        self.remainders[i.idx()] = Some(FreeRemainder::with_equation(equation));
        self.fast = false; // A foreign equation always disables fast handling for the net
      }
    }
    // Build null terminated pointer version of applicable for added speed.
    // ToDo: This optimization is dubious.
    let applicable_count = self.applicable.len();
    self.fast_applicable.resize_with(applicable_count, | | Vec::new());

    for i in 0..applicable_count {
      let live_set   = &self.applicable[i];
      let remainders = &mut self.fast_applicable[i];
      remainders.reserve(live_set.len());
      // remainders.resize_with(live_set.len() + 1, | | None);

      for rem in live_set.iter() {
        let free_remainder_ptr = FreeRemainderPtr::new(self.remainders[rem.idx()].as_mut().unwrap());
        remainders.push(free_remainder_ptr);
      }
    }
  }

  fn build_ternary_tree(
    &mut self,
    node_index     : &mut i32,
    triples        : &mut Vec<Triple>,
    first          : usize,
    last           : usize,
    default_subtree: i32,
    position       : SlotIndex,
    arg_index      : ArgIndex,
  ) {
    // Pick a middle element as the test symbol. If the sum of the first and last eligible indices
    // is odd we have a choice of middle elements and we try to break the tie in a smart way.
    let sum             = first + last;
    let mut test_symbol = sum / 2;
    if sum & 1 != 0 && self.more_important(&triples[test_symbol + 1].symbol, &triples[test_symbol].symbol) {
      test_symbol += 1;
    }

    // Fill out a new node.
    let i = *node_index as usize;
    *node_index += 1;
    self.net[i].position     = position;
    self.net[i].arg_index    = arg_index;
    self.net[i].symbol_index = triples[test_symbol].symbol.index_within_parent_module();
    self.net[i].slot         = triples[test_symbol].slot;
    self.net[i].equal        = triples[test_symbol].subtree;

    // If there are any symbols remaining to the left of the test symbol, build a subtree for them.
    if first < test_symbol {
      self.net[i].not_equal.less = *node_index;
      self.build_ternary_tree(node_index, triples, first, test_symbol - 1, default_subtree, SlotIndex::None, ArgIndex::None);
    } else {
      self.net[i].not_equal.less = default_subtree;
    }

    // If there are any symbols remaining to the right of the test symbol, build a subtree for them.
    if last > test_symbol {
      self.net[i].not_equal.greater = *node_index;
      self.build_ternary_tree(node_index, triples, test_symbol + 1, last, default_subtree, SlotIndex::None, ArgIndex::None);
    } else {
      self.net[i].not_equal.greater = default_subtree;
    }
  }

  /// Heuristic to decide which symbol is more important and thus should have the fastest matching.
  /// Returns true if first symbol is considered more important.
  ///
  /// The current heuristic favors free symbols over non-free symbols and high arity symbols over
  /// low arity symbols.
  fn more_important(&self, first: &SymbolPtr, second: &SymbolPtr) -> bool {
    let f = first.as_any().downcast_ref::<FreeSymbol>();
    let s = second.as_any().downcast_ref::<FreeSymbol>();

    match (f, s) {
      (Some(_), None) => true,
      (None, Some(_)) => false,
      _ => first.arity() > second.arity(),
    }
  }

  // region Rewriting related methods

  /// Traverses the discrimination net using the given subject term.
  /// Returns the index into `fast_applicable` if traversal succeeds.
  /// This is logic common to the `apply_replace_*` methods.
  fn find_fast_applicable_index(&mut self, subject: DagNodePtr) -> Option<i32> {
    if self.net.is_empty() {
      if !subject.symbol().arity().is_zero() {
        self.stack[0] = subject.get_arguments();
      }
      return Some(0);
    }

    let top_arg_array = subject.get_arguments();
    let mut net_idx = 0;
    let mut dag_node = top_arg_array[self.net[net_idx].arg_index.idx()];
    let mut symbol_index = dag_node.symbol().index_within_parent_module();
    self.stack[0] = top_arg_array;

    loop {
      let position: SlotIndex;
      let diff = symbol_index.idx() as isize
          - self.net[net_idx].symbol_index.idx() as isize;

      let next_index: i32;

      if diff != 0 {
        next_index = if diff < 0 {
          self.net[net_idx].not_equal.greater
        } else {
          self.net[net_idx].not_equal.less
        };

        if next_index <= 0 {
          return if next_index == 0 { None } else { Some(!next_index) };
        }

        net_idx = next_index as usize;
        position = self.net[net_idx].position;

        if !position.is_index() {
          continue;
        }
      } else {
        let slot = self.net[net_idx].slot;
        if slot.is_index() {
          self.stack[slot.idx()] = dag_node.get_arguments();
        }

        next_index = self.net[net_idx].equal;
        if next_index <= 0 {
          return Some(!next_index);
        }

        net_idx = next_index as usize;
        position = self.net[net_idx].position;
      }

      dag_node = self.stack[position.idx()][self.net[net_idx].arg_index.idx()];
      symbol_index = dag_node.symbol().index_within_parent_module();
    }
  }

  /// This is the inlined guard for `apply_replace` that provides a fast path in the case that
  /// the term cannot be applied.
  #[inline(always)]
  pub(crate) fn apply_replace(&mut self, subject: DagNodePtr, context: &mut RewritingContext) -> bool {
    if !self.applicable.is_empty() {
      self.apply_replace_aux(subject, context)
    } else {
      false
    }
  }

  /// Traverses the discrimination net for the given subject term and attempts to apply a matching rewrite rule.
  /// Returns true if a matching rule was successfully applied; otherwise, returns false.
  ///
  /// This general-purpose version handles symbols of any arity, including constants, and sets up the stack accordingly.
  pub fn apply_replace_aux(&mut self, subject: DagNodePtr, context: &mut RewritingContext) -> bool {
    let index = match self.find_fast_applicable_index(subject) {
      Some(i) => i,
      None    => return false,
    };

    // Now go through the sequence of remainders, trying to finish the match
    let fast_applicable_list = &mut self.fast_applicable[index as usize];
    for remainder in fast_applicable_list.iter_mut() {
      if remainder.fast_match_replace(subject, context, &mut self.stack) {
        return true;
      }
    }

    false
  }

  /// "Optimized" version of the the above that only works for unary, binary and ternary top symbols. The only
  /// difference is that this version assigns the subject's args to the stack unconditionally.
  // ToDo: It's hard to imagine that this is actually faster. Investigate.
  #[inline(always)]
  pub fn apply_replace_fast(&mut self, subject: DagNodePtr, context: &mut RewritingContext) -> bool {
    if self.applicable.is_empty() {
      false
    } else {
      // Optimization: pre-fill stack[0] unconditionally for unary, binary, ternary patterns
      self.stack[0] = subject.get_arguments();

      // Delegate to general logic
      self.apply_replace_aux(subject, context)
    }
  }

  /// This is the inlined guard for `apply_replace_no_owise` that provides a fast path in the case that
  /// the term cannot be applied.
  #[inline(always)]
  pub fn apply_replace_no_owise(&mut self, subject: DagNodePtr, context: &mut RewritingContext) -> bool {
    if self.applicable.is_empty() {
      false
    } else {
      self.apply_replace_no_owise_aux(subject, context)
    }
  }

  pub fn apply_replace_no_owise_aux(
    &mut self,
    subject: DagNodePtr,
    context: &mut RewritingContext,
  ) -> bool {
    let index = match self.find_fast_applicable_index(subject) {
      Some(i) => i,
      None => return false,
    };

    // Iterate over applicable remainders, stopping at first `owise`
    let fast_applicable_list = &mut self.fast_applicable[index as usize];
    for remainder in fast_applicable_list.iter_mut() {
      if remainder.equation.is_owise() {
        break;
      }
      if remainder.fast_match_replace(subject, context, &mut self.stack) {
        return true;
      }
    }

    false
  }

  // endregion Rewriting related methods
}
