/*!

A `Substitution` is a thin wrapper around a `Vec<&DagNode>`. It holds bindings between natural numbers and `DagNode`s
by placing a reference to the DagNode at the index of the number. Names are numbers, so these are bindings of names.

 From \[Eker 2003]:

 > For efficiency, the set of variable bindings at each stage in the
 > recursion in both simplify and build_hierarchy can be tracked by a
 > single global array indexed by small integers representing variables.

 */


use std::cmp::min;

use mod2_abs::NatSet;

use crate::{
  core::{
    LocalBindings,
    NarrowingVariableInfo,
    VariableInfo
  },
  api::{
    dag_node::DagNodePtr,
    variable_theory::VariableIndex
  }
};

pub type MaybeDagNode = Option<DagNodePtr>;

#[derive(Clone, Default)]
pub struct Substitution {
  bindings: Vec<MaybeDagNode>,

  // Todo: What is the purpose of copy_size?
  /*
  I think `copy_size` exists because the length of `bindings` might not reflect the "active" portion of the
  `Substitution`. The issue is that the `bindings` vector can never be truncated, because its `None` slots might be
  used elsewhere for construction purposes. So if some other `Substitution` with a smaller `bindings` vector is cloned
  into this one, the `copy_size` can be made smaller, but the `bindings` vector isn't truncated.

  The upshot is that we need a variable to track the size of `bindings` that is independent of its actual size.
  */
  copy_size: usize,
}

impl Substitution {
  #[inline(always)]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline(always)]
  pub fn with_capacity(n: usize) -> Self {
    let mut bindings = Vec::with_capacity(n);
    bindings.resize(n, None);

    Self { bindings, copy_size: n }
  }

  #[inline(always)]
  pub fn resize(&mut self, size: usize) {
    self.bindings.resize(size, None);
  }

  #[inline(always)]
  pub fn clear_first_n(&mut self, size: usize) {
    self.copy_size = size;

    for i in 0..min(size, self.bindings.len()) {
      self.bindings[i] = None;
    }

    if self.bindings.len() < size {
      self.bindings.resize(size, None);
    }
  }

  /// This getter takes a `usize` for the common case that we start with a `usize` index. Be careful that the `usize`
  /// wasn't converted from an `i32` that was `NONE`.
  #[inline(always)]
  pub fn value(&self, index: usize) -> MaybeDagNode {
    self.get(index as VariableIndex)
  }

  // Todo: Is this the best way to implement a getter? I think we did it this way so it returned a value.
  // ToDo: Should this take an VariableIndex?
  /// This getter takes an `i32` so it can check for negative indices, i.e. `NONE`.
  #[inline(always)]
  pub fn get(&self, index: VariableIndex) -> MaybeDagNode {
    assert!(index >= 0, "-ve index {}", index);
    assert!(
      (index as usize) < self.bindings.len(),
      "index too big {} vs {}",
      index,
      self.bindings.len()
    );

    // The asserts guarantee safety here.
    unsafe { self.bindings.get_unchecked(index as usize).clone() }
  }

  #[inline(always)]
  pub fn iter(&self) -> std::slice::Iter<Option<DagNodePtr>> {
    self.bindings.iter()
  }

  #[inline(always)]
  pub fn fragile_binding_count(&self) -> usize {
    self.copy_size
  }

  pub fn subtract(&self, original: &Substitution) -> Option<LocalBindings> {
    let mut local_bindings = LocalBindings::new();
    for (idx, (i, j)) in self.bindings.iter().zip(original.iter()).enumerate() {
      assert!(j.is_none() || i == j, "substitution inconsistency at index {}", idx);
      if let (Some(a), Some(b)) = (i, j) {
        if !a.addr_eq(*b) {
          local_bindings.add_binding(idx as VariableIndex, *a);
        }
      }
    }

    if local_bindings.len() > 0 {
      Some(local_bindings)
    } else {
      None
    }
  }

  /*
  Actually, I think these are implemented on `LocalBindings`

   pub fn assert(&self, solution: &Substitution) {
     // Todo: Implement assert
   }


   pub fn retract(&self, solution: &Substitution) {
     // Todo: Implement retract
   }
  */

  #[inline(always)]
  pub fn bind(&mut self, index: VariableIndex, maybe_value: Option<DagNodePtr>) {
    assert!(index >= 0, "Negative index {}", index);
    assert!(
      (index as usize) < self.bindings.len(),
      "Index too big {} vs {}",
      index,
      self.bindings.len()
    );

    self.bindings[index as usize] = maybe_value;
  }

  #[inline(always)]
  pub fn copy_from_substitution(&mut self, original: &Substitution) {
    assert_eq!(self.copy_size, original.copy_size);

    if self.copy_size > 0 {
      self.bindings = original.bindings.clone();
    }
  }

  #[inline(always)]
  pub fn finished(&mut self) {
    self.copy_size = 0;
  }
}


// More specialized print functions for substitutions. These are used in narrowing.rs, trace_variant_narrowing_step in
// rewrite_context.rs.

pub fn print_substitution_dag(substitution: &[DagNodePtr], variable_info: &NarrowingVariableInfo) {
  for (i, var) in variable_info.iter() {
    let binding = substitution[i];

    println!("{} --> {}", var, binding);
  }
}

pub fn print_substitution_narrowing(substitution: &Substitution, variable_info: &NarrowingVariableInfo) {
  let variable_count = substitution.fragile_binding_count();

  for i in 0..variable_count {
    let var = variable_info.index_to_variable(i).unwrap();
    let binding = substitution.value(i);
    assert!(binding.is_some(), "A variable is bound to None. This is a bug.");
    let binding = binding.unwrap();
    println!("{} --> {}", var, binding);
  }
}

pub fn print_substitution(substitution: &Substitution, var_info: &VariableInfo) {
  print_substitution_with_ignored(substitution, var_info, &NatSet::default())
}

pub fn print_substitution_with_ignored(substitution: &Substitution, var_info: &VariableInfo, ignored_indices: &NatSet) {
  let variable_count = var_info.real_variable_count();
  let mut printed_variable = false;
  for i in 0..variable_count {
    if ignored_indices.contains(i) {
      continue;
    }
    let var = var_info.index_to_variable(i as VariableIndex);
    debug_assert!(var.is_some(), "null variable");
    let var = var.unwrap();
    print!("{} --> ", var);

    match substitution.value(i) {
      None => {
        println!("(unbound)");
      }
      Some(binding) => {
        println!("{}", binding);
        printed_variable = true;
      }
    }
  }
  if !printed_variable {
    println!("empty substitution");
  }
}
