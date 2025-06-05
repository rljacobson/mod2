/*!



*/

use std::{
  fmt::Write
};

use mod2_abs::{numeric::{
  traits::{One, Zero},
  BigInt,
}, debug, NatSet, tracing::warn, error, HashSet, HashMap, warning};

use crate::{
  core::{
    sort::{
      SortIndex,
      SortPtr,
      kind::KindPtr,
    },
    symbol::op_declaration::{ConstructorStatus, OpDeclaration}
  },
  api::{
    symbol::SymbolPtr,
    Arity
  },
};


pub type BxSortTable = Box<SortTable>;

// ToDo: Most of these vectors are likely to be small. Benchmark with tiny_vec.
#[derive(PartialEq, Eq)]
pub struct SortTable {
  arity                    : Arity,              // possibly `Any`, etc.
  arg_count                : i16,
  op_declarations          : Vec<OpDeclaration>,
  arg_kinds                : Vec<KindPtr>,       // "component vector"
  pub sort_diagram         : Vec<SortIndex>,
  single_non_error_sort    : Option<SortPtr>,    // if we can only generate one non-error sort
  constructor_diagram      : Vec<SortIndex>,
  maximal_op_decl_set_table: Vec<NatSet>,        // indices of maximal op decls with range <= each sort
}

impl Default for SortTable {
  fn default() -> Self {
    Self {
      arity                    : Arity::Unspecified,
      arg_count                : 0,
      op_declarations          : Vec::new(),
      arg_kinds                : Vec::new(),
      sort_diagram             : Vec::new(),
      single_non_error_sort    : None,
      constructor_diagram      : Vec::new(),
      maximal_op_decl_set_table: Vec::new(),
    }
  }
}

impl SortTable {
  pub fn new() -> Self {
    SortTable::default()
  }


  /// Is the symbol strictly a constructor (non constructor)? Used to determine if
  /// every member of the kind can share a single constructor.
  #[inline(always)]
  pub fn constructor_status(&self) -> ConstructorStatus {
    let mut constructor_status = ConstructorStatus::Unspecified;
    for declaration in &self.op_declarations {
      constructor_status |= declaration.is_constructor;
    }
    constructor_status
  }

  #[inline(always)]
  pub fn arity(&self) -> Arity {
    if self.arg_count == 0 {
      assert!(
        match self.arity {
          Arity::Value(v) if v > 0 => false,
          _ => true
        }
      );
      self.arity
    } else {
      self.arg_count.into()
    }
  }

  #[inline(always)]
  pub fn get_maximal_op_decl_set(&mut self, target: SortPtr) -> &NatSet {
    if self.maximal_op_decl_set_table.is_empty() {
      self.compute_maximal_op_decl_set_table();
    }
    &self.maximal_op_decl_set_table[target.index_within_kind.idx_unchecked()]
  }

  #[inline(always)]
  pub fn special_sort_handling(&self) -> bool {
    self.sort_diagram.is_empty()
  }

  #[inline(always)]
  pub fn add_op_declaration(&mut self, op_declaration: OpDeclaration) {
    if self.op_declarations.is_empty() {
      self.arg_count = op_declaration.arity();
    } else {
      assert_eq!(
        (op_declaration.len() - 1) as i16,
        self.arg_count,
        "bad domain length of {} instead of {}",
        (op_declaration.len() - 1) as i16,
        self.arg_count
      );
    }

    self.op_declarations.push(op_declaration);
  }

  #[inline(always)]
  pub fn get_op_declarations(&self) -> &Vec<OpDeclaration> {
    &self.op_declarations
  }

  #[inline(always)]
  pub fn range_kind(&self) -> KindPtr {
    // ToDo: Is this function fallible? Should it return `Option<KindPtr>`?
    //       If this is only ever called after `Module::compute_kind_closures()`, this is safe.
    assert!(!self.op_declarations.is_empty(), "cannot get range kind for symbol with no op declarations");
    unsafe { (&self.op_declarations[0])[self.arg_count as usize].kind.unwrap_unchecked() }
  }

  /// If an operator has been declared with multiple range sort, this
  /// function just returns the first, which is good enough for some
  /// purposes.
  #[inline(always)]
  pub fn get_range_sort(&self) -> SortPtr {
    assert!(!self.op_declarations.is_empty(), "cannot get range sort for symbol with no op declarations");
    (&self.op_declarations[0])[self.arg_count as usize]
  }

  #[inline(always)]
  pub fn domain_component(&self, idx: usize) -> KindPtr {
    assert!(
      idx < self.op_declarations.len(),
      "cannot get domain kind {} for symbol with {} args",
      idx + 1,
      self.op_declarations.len()
    );
    unsafe { (&self.op_declarations[0])[idx].kind.unwrap_unchecked() }
  }

  // #[inline(always)]
  // pub fn domain_components_iter(&self) -> Box<dyn Iterator<Item = KindPtr>> {
  //   Box::new(
  //     (&self.op_declarations[0])
  //         .iter()
  //         .map(|v| unsafe{ v.kind.unwrap_unchecked() }),
  //   )
  // }

  #[inline(always)]
  pub fn get_single_non_error_sort(&self) -> Option<SortPtr> {
    self.single_non_error_sort.clone()
  }

  #[inline(always)]
  pub fn traverse(&self, position: usize, sort_index: SortIndex) -> SortIndex {
    self.sort_diagram[position + sort_index.idx_unchecked()]
  }

  #[inline(always)]
  pub fn constructor_traverse(&self, position: usize, sort_index: SortIndex) -> SortIndex {
    self.constructor_diagram[position + sort_index.idx_unchecked()]
  }

  pub fn domain_subsumes(&self, subsumer: usize, victim: usize) -> bool {
    let s = &self.op_declarations[subsumer];
    let v = &self.op_declarations[victim];

    for i in 0..self.arg_count as usize {
      if !v[i].leq(s[i]) {
        return false;
      }
    }
    true
  }

  pub fn compute_maximal_op_decl_set_table(&mut self) {
    let range             = self.range_kind();
    let sort_count        = range.sort_count();
    let declaration_count = self.op_declarations.len();

    self.maximal_op_decl_set_table
        .resize(sort_count, NatSet::new());

    for i in 0..sort_count {
      let target = range.sort( SortIndex::try_from(i).unwrap() );

      for j in 0..declaration_count {
        if (&self.op_declarations[j])[self.arg_count as usize].leq(target) {
          for k in 0..j {
            if self.maximal_op_decl_set_table[i].contains(k) {
              if self.domain_subsumes(k, j) {
                continue;
              } else if self.domain_subsumes(j, k) {
                self.maximal_op_decl_set_table[i].remove(k);
              }
            }
          }

          self.maximal_op_decl_set_table[i].insert(j);
        }
      }
    }
  }

  /// Called from `Module::close_theory()`. The `symbol_ptr` is the owner and is passed for error and debug logging.
  pub fn compile_op_declaration(&mut self, symbol_ptr: SymbolPtr) {
    debug_assert!(self.op_declarations.len() > 0);
    self.arg_kinds.reserve((self.arg_count + 1) as usize);

    for (i, arg) in self.op_declarations[0].sort_spec.iter().enumerate() {
      let kind = unsafe{ arg.kind.unwrap_unchecked() };

      // Check that components really do agree for subsort overloaded operator declarations
      for other_op_decl in self.op_declarations.iter().skip(1){
        let other_op_kind = unsafe{ other_op_decl.sort_spec[i].kind.unwrap_unchecked() };
        if kind != other_op_kind {
          error!(0,
            "Sort declarations for operator {} disagree on the sort component for argument {}",
            symbol_ptr,
            i + 1
          );
        }
      }

      self.arg_kinds.push(kind);
    }

    self.build_sort_diagram(symbol_ptr);
    if self.constructor_status() == ConstructorStatus::Complex {
      // self.build_constructor_diagram();
    }
  }

  /// Builds the sort diagram for this operator, encoding all valid combinations of argument
  /// sorts and their corresponding result sorts. This precomputes a table used at runtime for
  /// efficient sort checking and inference during term rewriting. Handles both polymorphic
  /// and overloaded operator declarations, and detects sort ambiguities when they arise.
  fn build_sort_diagram(&mut self, symbol_ptr: SymbolPtr) {
    let nr_declarations    = self.op_declarations.len();
    let mut current_states = vec![NatSet::new()];
    let all                = &mut current_states[0];

    // Initialize with all declarations in reverse order
    for i in (0..nr_declarations).rev() {
      all.insert(i);
    }

    if self.arg_count == 0 {
      let (sort_index, unique) = self.find_min_sort_index(all);
      assert!(unique, "sort declarations for constant do not have a unique least sort.");
      self.sort_diagram.push(sort_index);
      self.single_non_error_sort = Some(self.arg_kinds[0].sort(sort_index));
      return;
    }

    let mut single_non_error_sort_index = SortIndex::UNINITIALIZED;
    let mut next_states                 = Vec::new();
    let mut current_base                = 0;
    let mut bad_terminals               = HashSet::new();

    for i in 0..self.arg_count as usize {
      let component         = self.arg_kinds[i];
      let nr_sorts          = component.sort_count();
      let nr_current_states = current_states.len();

      let next_base = current_base + nr_sorts * nr_current_states;
      self.sort_diagram.resize(next_base, SortIndex::ZERO);

      let nr_next_sorts = if i == (self.arg_count - 1) as  usize {
        0
      } else {
        self.arg_kinds[i + 1].sort_count()
      };

      for j in 0..nr_sorts {
        let s = component.sort(j.try_into().unwrap());
        let mut viable = NatSet::new();

        for (k, decl) in self.op_declarations.iter().enumerate() {
          if s.leq(decl.sort_spec[i]) {
            viable.insert(k);
          }
        }

        for k in 0..nr_current_states {
          let mut next_state = viable.intersection(&current_states[k]);
          let index          = current_base + k * nr_sorts + j;

          if nr_next_sorts == 0 {
            let (sort_index, unique) = self.find_min_sort_index(&next_state);
            self.sort_diagram[index] = sort_index;
            if !unique {
              bad_terminals.insert(index.try_into().unwrap());
            }
            if sort_index.is_positive() {
              if single_non_error_sort_index == SortIndex::UNINITIALIZED {
                single_non_error_sort_index = sort_index;
              } else if single_non_error_sort_index != sort_index {
                single_non_error_sort_index = SortIndex::IMPOSSIBLE;
              }
            }
          } else {
            self.minimize(&mut next_state, i + 1);
            let state_num            = SortTable::find_state_number(&mut next_states, next_state);
            let new_state            = next_base + nr_next_sorts * state_num;
            self.sort_diagram[index] = new_state.try_into().unwrap();
          }
        }
      }

      std::mem::swap(&mut current_states, &mut next_states);
      next_states.clear();
      current_base = next_base;
    }

    if single_non_error_sort_index.is_positive() {
      self.single_non_error_sort = Some(
        self.arg_kinds[self.arg_count as usize].sort(single_non_error_sort_index),
      );
    }

    if !bad_terminals.is_empty() {
      self.sort_error_analysis(true, &bad_terminals);
    }

    debug!(4, "sort table for {} has {} entries", symbol_ptr, self.sort_diagram.len());
  }


  /// Given a set of operator declarations, finds the minimal (most specific) result sort
  /// that is consistent with all declarations in the set. The index of the minimal sort
  /// and a bool to indicate whether the minimal sort is unambiguous. Used to determine
  /// the result sort for a given combination of argument sorts in the sort diagram.
  fn find_min_sort_index(&self, state: &NatSet) -> (SortIndex, bool) {
    let arg_count = self.arg_count as usize;

    // Start with the error sort
    let mut min_sort   = self.arg_kinds[arg_count].sort(SortIndex::ERROR);
    let mut inf_so_far = min_sort.leq_sorts.clone();

    for i in state.iter() {
      let range_sort      = self.op_declarations[i].sort_spec[arg_count];
      let range_leq_sorts = &range_sort.leq_sorts;
      inf_so_far          = inf_so_far.intersection(range_leq_sorts);

      // If the intersection is exactly equal to the current range's leq
      // sorts, it means the current range sort is ≤ all seen so far.
      //
      // This test only succeeds if rangeSort is less than or equal to the range
      // sorts of the previous declarations in the state. Thus, in the case of
      // a non-preregular operator we break in favor of the earlier declaration.
      if inf_so_far == *range_leq_sorts {
        min_sort = range_sort;
      }
    }

    let unique = inf_so_far == min_sort.leq_sorts;
    (min_sort.index_within_kind, unique)
  }

  /// Removes redundant operator declarations from the given set by eliminating any
  /// declaration that is partially subsumed by an earlier one, relative to the specified
  /// argument position. This reduces the number of states needed in the sort diagram
  /// by collapsing equivalent or more general cases during diagram construction.
  fn minimize(&self, alive: &mut NatSet, arg_nr: usize) {
    // Skip if the set is empty
    if alive.is_empty() {
      return;
    }

    let min = alive.min_value().unwrap();
    let max = alive.max_value().unwrap();

    for i in min..=max {
      if alive.contains(i) {
        for j in min..=max {
          if j != i && alive.contains(j) && self.partially_subsumes(i, j, arg_nr) {
            alive.remove(j);
          }
        }
      }
    }
  }

  /// Determines whether one operator declaration (`subsumer`) partially subsumes
  /// another (`victim`) at or beyond the given argument position (`arg_nr`). A declaration
  /// subsumes another if its result sort is less than or equal to the other's, and each
  /// corresponding argument sort from `arg_nr` onward is greater than or equal to the
  /// other's. Used in minimization to eliminate redundant declarations.
  fn partially_subsumes(&self, subsumer: usize, victim: usize, arg_idx: usize) -> bool {
    let arg_count      = self.arg_count as usize;
    let subsumer_sorts = &self.op_declarations[subsumer].sort_spec;
    let victim_sorts   = &self.op_declarations[victim].sort_spec;

    // Check the range sort
    if !subsumer_sorts[arg_count].leq(victim_sorts[arg_count]) {
      return false;
    }
    // Check the domain sorts
    for i in arg_idx..arg_count {
      if !victim_sorts[i].leq(subsumer_sorts[i]) {
        return false;
      }
    }

    true
  }

  /// Finds the index of a given state (`state`) in the existing set of states (`state_set`).
  /// If the state is already present, returns its index. Otherwise, appends it to the set
  /// and returns the new index. This is used during sort diagram construction to assign
  /// unique indices to distinct sort state combinations.
  fn find_state_number(state_set: &mut Vec<NatSet>, state: NatSet) -> usize {
    for (i, existing) in state_set.iter().enumerate() {
      if *existing == state {
        return i;
      }
    }
    state_set.push(state);
    state_set.len() - 1
  }


  /// Performs consistency error analysis on the sort or constructor diagram after detecting
  /// ambiguous or conflicting sort declarations. It builds a spanning tree from the sort
  /// diagram to trace sort tuples that lead to errors, computes how many such tuples exist,
  /// and identifies the first encountered problematic tuple to provide meaningful diagnostics.
  fn sort_error_analysis(&self, prereg_problem: bool, bad_terminals: &HashSet<SortIndex>) {
    // First we build a spanning tree with a path count, parent node and sort index (from parent)
    // for each node. Nonterminal nodes are named by their start index in the diagram vector
    // while terminals are named by the absolute index of the terminal in the diagram vector.

    /// Represents a node in the sort error analysis spanning tree.
    #[derive(Default)]
    struct Node {
      path_count: BigInt,
      parent    : SortIndex,
      sort_index: SortIndex,
    }

    let diagram = if prereg_problem { &self.sort_diagram } else { &self.constructor_diagram };
    let nr_args = self.arg_count as usize;

    let mut spanning_tree: HashMap<usize, Node> = HashMap::new();
    spanning_tree.insert(0, Node {
      path_count: BigInt::one(),
      parent    : SortIndex::UNKNOWN,
      sort_index: SortIndex::UNKNOWN,
    });

    let mut product       = BigInt::one();
    let mut bad_count     = BigInt::zero();
    let mut first_bad     = vec![SortIndex::ZERO; nr_args];
    let mut current_nodes = HashSet::new();

    current_nodes.insert(SortIndex::ZERO);

    for i in 0..nr_args {
      let component       = self.arg_kinds[i];
      let nr_sorts        = component.sort_count();
      let mut next_nodes  = HashSet::new();
      product            *= nr_sorts;

      for &parent in &current_nodes {
        let path_count = spanning_tree[&parent.idx_unchecked()].path_count.clone();

        for k in 0..nr_sorts {
          let index = parent + k;

          if i == nr_args - 1 {
            // Terminal node
            if bad_terminals.contains(&index) {
              if bad_count.is_zero() {
                bad_count = path_count.clone();
                first_bad[nr_args - 1] = k.try_into().unwrap();

                let mut n = parent;
                for l in (0..nr_args - 1).rev() {
                  if let Some(node) = spanning_tree.get(&n.idx_unchecked()) {
                    first_bad[l] = node.sort_index;
                    n = node.parent;
                  } else {
                    panic!("missing node in spanning tree");
                  }
                }
              } else {
                bad_count += path_count.clone();
              }
            }
          } else {
            // Non-terminal node
            let target = diagram[index.idx_unchecked()];
            let entry  = spanning_tree.entry(target.idx_unchecked()).or_insert_with(Node::default);

            if entry.path_count.is_zero() {
              entry.path_count = path_count.clone();
              entry.parent     = parent;
              entry.sort_index = k.try_into().unwrap();
              next_nodes.insert(target);
            } else {
              entry.path_count += path_count.clone();
            }
          }
        }
      }

      current_nodes = next_nodes;
    }

    // Emit warning
    let kind  = if prereg_problem { "sort" } else { "constructor" };
    let check = if prereg_problem { "preregularity" } else { "constructor consistency" };

    warning!(
      0,
      "{} declarations for operator symbol failed {} check on {} out of {} sort tuples. First such tuple is ({}).",
      kind,
      check,
      bad_count,
      product,
      first_bad.iter()
               .enumerate()
               .map(|(i, &idx)| format!("{}", self.arg_kinds[i].sort(idx.try_into().unwrap())))
               .collect::<Vec<_>>()
               .join(", ")
    );
  }

  /// Usage:
  ///     let mut out = String::new();
  ///     sort_table.dump_sort_diagram(&mut out, 2).unwrap();
  ///     println!("{}", out);
  #[cfg(feature = "debug")]
  pub fn dump_sort_diagram(&self, f: &mut impl Write, indent_level: usize) -> std::fmt::Result {
    if self.special_sort_handling() {
      return Ok(());
    }

    writeln!(f, "{}Begin{{SortDiagram}}", " ".repeat(indent_level))?;
    let indent_level = indent_level + 2;
    let mut nodes: HashSet<SortIndex> = HashSet::new();
    nodes.insert(SortIndex::ZERO);

    let range = self.arg_kinds[self.arg_count as usize]; // Result component

    if self.arg_count == 0 {
      let target = self.sort_diagram[0];
      writeln!(
        f,
        "{}node 0 -> sort {} ({})",
        " ".repeat(indent_level - 1),
        target,
        range.sort(target)
      )?;
      return Ok(());
    }

    for i in 0..(self.arg_count as usize) {
      let component = self.arg_kinds[i];
      let nr_sorts = component.sort_count();
      let mut next_nodes = HashSet::new();

      for &node in &nodes {
        writeln!(
          f,
          "{}Node {} (testing argument {})",
          " ".repeat(indent_level - 1),
          node,
          i
        )?;

        for k in 0..nr_sorts {
          let index = node + k;
          let target = self.sort_diagram[index.idx_unchecked()];

          write!(
            f,
            "{}sort {} ({}) -> ",
            " ".repeat(indent_level),
            k,
            component.sort(k.try_into().unwrap())
          )?;

          if i == self.arg_count as usize - 1 {
            writeln!(
              f,
              "sort {} ({})",
              target,
              range.sort(target)
            )?;
          } else {
            writeln!(f, "node {}", target)?;
            next_nodes.insert(target);
          }
        }
      }

      nodes = next_nodes;
    }

    writeln!(f, "{}End{{SortDiagram}}", " ".repeat(indent_level - 2))?;
    Ok(())
  }
}
