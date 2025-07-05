use std::{
  any::Any,
  cmp::Ordering,
  fmt::{Display, Formatter, Pointer},
  ops::Deref
};

use mod2_abs::{hash::hash2 as term_hash, impl_as_any_ptr_fns, NatSet, PartialOrdering};

use crate::{
  api::{
    automaton::{
      BxLHSAutomaton,
      LHSAutomaton,
      RHSAutomaton
    },
    dag_node::{
      arg_to_node_vec,
      DagNode,
      DagNodePtr,
      DagNodeVector,
    },
    dag_node_cache::DagNodeCache,
    free_theory::{
      free_automata::{FreeLHSAutomaton, FreeRHSAutomaton},
      FreeDagNode,
      FreeOccurrence,
      FreeOccurrences
    },
    symbol::SymbolPtr,
    term::{
      BxTerm,
      Term,
      TermPtr
    },
    variable_theory::VariableTerm,
  },
  core::{
    automata::RHSBuilder,
    dag_node_core::{
      DagNodeCore,
      DagNodeFlag,
    },
    format::{
      FormatStyle,
      Formattable,
    },
    substitution::Substitution,
    symbol::SymbolTranslationMap,
    term_core::TermCore,
    ArgIndex,
    SlotIndex,
    TermBag,
    VariableIndex,
    VariableInfo,
  },
  impl_display_debug_for_formattable,
  HashType,
};


pub struct FreeTerm{
  core                 : TermCore,
  pub args             : Vec<BxTerm>,
  pub(crate) slot_index: SlotIndex,
}

impl FreeTerm {
  pub fn new(symbol: SymbolPtr, args: Vec<BxTerm>) -> Self {
    Self {
      core      : TermCore::new(symbol),
      args,
      slot_index: SlotIndex::None,
    }
  }
}

impl Formattable for FreeTerm {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    match style {
      FormatStyle::Simple => {
        self.symbol().repr(f, style)?;
      }

      FormatStyle::Debug | _ => {
        write!(f, "free<")?;
        self.symbol().repr(f, style)?;
        write!(f, ">")?;
      }
    }

    if !self.args.is_empty() {
      let mut args = self.args.iter();
      write!(f, "(")?;
      args.next().unwrap().repr(f, style)?;
      for arg in args {
        write!(f, ", ")?;
        arg.repr(f, style)?;
      }
      write!(f, ")")?;
    }

    Ok(())
  }
}

impl_display_debug_for_formattable!(FreeTerm);


impl Term for FreeTerm {
  //region Representation and Reduction Methods
  // impl_as_any_ptr_fns!(Term, FreeTerm);
  fn as_any(&self) -> &dyn std::any::Any { self }
  fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
  fn as_ptr(&self) -> TermPtr {
    TermPtr::new(self as *const dyn Term as *mut dyn Term)
  }

  fn structural_hash(&self) -> HashType {
    self.core().hash_value
  }

  /// In sync with `FreeDagNode::structural_hash()`
  fn normalize(&mut self, full: bool) -> (Option<BxTerm>, bool, HashType) {
    let mut changed   : bool     = false;
    let mut hash_value: HashType = self.symbol().hash();

    // We want to be able to mutate args while iterating.
    for idx in 0..self.args.len() {
      let (maybe_new_child, child_changed, child_hash) = self.args[idx].normalize(full);
      if let Some(new_child) = maybe_new_child {
        // In `Maude`, term nodes that change must delete themselves. Here, we overwrite the previous value, and the
        // compiler makes sure it's dropped.
        self.args[idx] = new_child;
      }

      changed = changed || child_changed;
      hash_value = term_hash(hash_value, child_hash);
    }

    self.core_mut().hash_value = hash_value;

    (None, changed, hash_value)
  }

  fn deep_copy_aux(&self) -> BxTerm {
    let symbol = self.core().symbol();
    let args   = self.args.iter().map(|t| t.deep_copy()).collect();

    symbol.make_term(args)
  }

  // endregion

  fn core(&self) -> &TermCore {
    &self.core
  }

  fn core_mut(&mut self) -> &mut TermCore {
    &mut self.core
  }

  fn iter_args(&self) -> Box<dyn Iterator<Item=TermPtr> + '_> {
    Box::new(self.args.iter().map(| t | t.as_ptr()))
  }

  // region Comparison Methods

  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering {
    assert_eq!(&self.symbol(), &other.symbol(), "symbols differ");

    if let Some(other) = other.as_any().downcast_ref::<FreeTerm>() {
      for (arg_self, arg_other) in self.args.iter().zip(other.args.iter()) {
        let r = arg_self.compare(arg_other.deref());
        if r.is_ne() {
          return r;
        }
      }
      Ordering::Equal
    } else {
      unreachable!("Could not downcast Term to FreeTerm. This is a bug.")
    }
  }

  fn compare_dag_arguments(&self, other: DagNodePtr) -> Ordering {
    assert_eq!(self.symbol(), other.symbol(), "symbols differ");
    if let Some(other) = other.as_any().downcast_ref::<FreeDagNode>() {
      for (arg_self, arg_other) in self.args.iter().zip(other.iter_args()) {
        let r = arg_self.compare_dag_node(arg_other);
        if r.is_ne() {
          return r;
        }
      }
      Ordering::Equal
    } else {
      unreachable!("Could not downcast Term to FreeTerm. This is a bug.")
    }
  }

  // ToDo: This method makes no use of partial_substitution except for `partial_compare_unstable` in `VariableTerm`.
  fn partial_compare_arguments(&self, partial_substitution: &mut Substitution, other: DagNodePtr) -> Option<Ordering> {
    assert!(self.symbol().compare(other.symbol().deref()).is_eq(), "symbols differ");

    // ToDo: Maude's implementation does a static cast and doesn't check that they have the same number of arguments.
    //       Which implementation should we use here?


    // for (term_arg, dag_arg) in self.iter_args().zip(other.iter_args()) {
    //   let r = term_arg.partial_compare(partial_substitution, dag_arg.deref());
    //   if r?.is_ne() {
    //     return r;
    //   }
    // }
    // Some(Ordering::Equal)


    if let Some(da) = other.as_any().downcast_ref::<FreeDagNode>() {
      for (term_arg, dag_arg) in self.args.iter().zip(da.iter_args()) {
        let r = term_arg.partial_compare(partial_substitution, dag_arg);
        if r != PartialOrdering::Equal {
          return r;
        }
      }

      if self.args.len() < da.len() { return PartialOrdering::Less }
      else if self.args.len() > da.len() { return PartialOrdering::Greater }

      PartialOrdering::Equal
    } else {
      unreachable!(
        "{}:{}: Could not downcast to FreeDagNode. This is a bug.",
        file!(),
        line!()
      )
    }
  }

  // endregion

  #[allow(private_interfaces)]
  fn dagify_aux(&self, node_cache: &mut DagNodeCache) -> DagNodePtr {
    let mut new_node = FreeDagNode::new(self.symbol());

    for arg in self.args.iter() {
      let node = arg.dagify(node_cache);
      new_node.insert_child(node);
    }

    new_node
  }

  // region Compiler-related
  #[inline(always)]
  fn compile_lhs_aux(
    &mut self,
    _match_at_top : bool,
    variable_info : &VariableInfo,
    bound_uniquely: &mut NatSet,
  ) -> (BxLHSAutomaton, bool) {
    // We bin the arg terms according to the following categories.
    // First gather all symbols lying in or directly under free skeleton.
    let mut free_symbols  = FreeOccurrences::new();
    let mut other_symbols = FreeOccurrences::new();
    // See if we can fail on the free skeleton.
    self.scan_free_skeleton(&mut free_symbols, &mut other_symbols, SlotIndex::None, ArgIndex::None);

    // Now classify occurrences of non Free-Theory symbols into 4 types
    let mut bound_variables     = FreeOccurrences::new(); // guaranteed bound when matched against
    let mut uncertain_variables = FreeOccurrences::new(); // status when matched against uncertain
    let mut ground_aliens       = FreeOccurrences::new(); // ground alien subterms
    let mut non_ground_aliens   = FreeOccurrences::new(); // non-ground alien subterms


    for mut occurrence in other_symbols {
      if let Some(v) = occurrence.try_downcast_term_mut::<VariableTerm>() {
        assert_ne!(v.index, VariableIndex::None, "index is none");
        let index: VariableIndex = v.index;
        assert!(index > 100, "index too big");

        if bound_uniquely.contains(index.idx()) {
          bound_variables.push(occurrence);
        } else {
          bound_uniquely.insert(index.idx());
          uncertain_variables.push(occurrence);
        }
      } else {
        let term: &dyn Term = occurrence.term();
        if term.ground() {
          ground_aliens.push(occurrence);
        } else {
          non_ground_aliens.push(occurrence);
        }
      }
    }

    // Now reorder the non-ground alien args in an order most likely to fail fast.
    // Now we have to find a best sequence in which to match the
    // non-ground alien subterms and generate subautomata for them

    let mut best_sequence     = ConstraintPropagationSequence::default();
    let mut sub_automata      = Vec::with_capacity(non_ground_aliens.len());
    let mut subproblem_likely = false;

    if non_ground_aliens.len() > 0 {
      Self::find_constraint_propagation_sequence(&mut non_ground_aliens, bound_uniquely, &mut best_sequence);

      for &sequence_index in &best_sequence.sequence {
        let (automata, spl): (BxLHSAutomaton, bool) =
            non_ground_aliens[sequence_index.idx()]
                .term_mut()
                .compile_lhs(false, variable_info, bound_uniquely);
        sub_automata.push(Some(automata));
        subproblem_likely = subproblem_likely || spl;
      }
      assert!(*bound_uniquely == best_sequence.bound, "Bound clash. This is a bug.");
    }

    let automaton: Box<dyn LHSAutomaton> = FreeLHSAutomaton::new(
      free_symbols,
      uncertain_variables,
      bound_variables,
      ground_aliens,
      non_ground_aliens,
      best_sequence.sequence,
      sub_automata,
    );

    (automaton, subproblem_likely)
  }

  /// The theory-dependent part of `compile_rhs` called by `term_compiler::compile_rhs(…)`. Returns
  /// the `save_index`. Maude's `compileRhs2`
  #[inline(always)]
  fn compile_rhs_aux(
    &mut self,
    rhs_builder    : &mut RHSBuilder,
    variable_info  : &mut VariableInfo,
    available_terms: &mut TermBag,
    eager_context  : bool,
  ) -> VariableIndex {
    let mut max_arity = 0;
    let mut free_variable_count = 1;
    self.compile_rhs_aliens(
      rhs_builder,
      variable_info,
      available_terms,
      eager_context,
      &mut max_arity,
      &mut free_variable_count,
    );

    let mut automaton: Box<dyn RHSAutomaton> =
        FreeRHSAutomaton::with_arity_and_free_variable_count(max_arity, free_variable_count);

    let index = self.compile_into_automaton(
      automaton.as_mut(),
      rhs_builder,
      variable_info,
      available_terms,
      eager_context,
    );

    rhs_builder.add_rhs_automaton(automaton);
    index
  }


  fn analyse_constraint_propagation(&mut self, bound_uniquely: &mut NatSet) {
    // First gather all symbols lying in or directly under free skeleton.
    let mut free_symbols  = Vec::new();
    let mut other_symbols = Vec::new();
    self.scan_free_skeleton(&mut free_symbols, &mut other_symbols, SlotIndex::None, ArgIndex::None);

    // Now extract the non-ground aliens and update BoundUniquely with variables
    // that lie directly under the free skeleton and thus will receive an unique binding.
    let mut non_ground_aliens = Vec::new();
    for occurrence in &mut other_symbols {
      let t = occurrence.term_mut();
      if let Some(variable_term) = t.as_any_mut().downcast_mut::<VariableTerm>() {
        bound_uniquely.insert(variable_term.index.idx());
      } else if !t.ground() {
        non_ground_aliens.push(occurrence.clone());
      }
    }

    if !non_ground_aliens.is_empty() {
      // debug_advisory(&format!(
      //   "FreeTerm::analyseConstraintPropagation() : looking at {} and saw {} nonground aliens",
      //   self,
      //   non_ground_aliens.len()
      // ));

      // Now we have to find a best sequence in which to match the non-ground alien subterms. Sequences that pin down
      // unique values for variables allow those values to be propagated.
      let mut best_sequence = ConstraintPropagationSequence::default();

      Self::find_constraint_propagation_sequence_helper(
        &mut non_ground_aliens,
        &mut vec![],
        &bound_uniquely,
        0,
        &mut best_sequence,
      );

      bound_uniquely.union_in_place(&best_sequence.bound);
    }
  }

    #[inline(always)]
    fn find_available_terms_aux(&self, available_terms: &mut TermBag, eager_context: bool, at_top: bool) {
      if self.ground() {
        return;
      }

      let arg_count = self.args.len();
      let symbol    = self.symbol();

      if at_top {
        for i in 0..arg_count {
          self.args[i].find_available_terms_aux(
            available_terms,
            eager_context && symbol.eager_argument(i),
            false,
          );
        }
      } else {
        available_terms.insert_matched_term(self.as_ptr(), eager_context);
        for i in 0..arg_count {
          self.args[i].find_available_terms_aux(
            available_terms,
            eager_context && symbol.evaluated_argument(i),
            false,
          );
        }
      }
    }
    // endregion
}


// Only used locally. Other theories will have their own local version.
#[derive(Default)]
struct ConstraintPropagationSequence {
  sequence   : Vec<ArgIndex>,
  bound      : NatSet,
  cardinality: i32,
}


impl FreeTerm {


  /// Traverse the free skeleton, calling compile_rhs() on all non-free subterms.
  pub fn compile_rhs_aliens(
    &mut self,
    rhs_builder        : &mut RHSBuilder,
    variable_info      : &mut VariableInfo,
    available_terms    : &mut TermBag,
    eager_context      : bool,
    max_arity          : &mut u32,
    free_variable_count: &mut u32,
  ) {
    let arg_count = self.args.len() as u32;
    if arg_count > *max_arity {
      *max_arity = arg_count;
    }
    let symbol = self.symbol();
    for i in 0..arg_count as usize {
      let arg_eager = eager_context && symbol.eager_argument(i);
      let term = &mut self.args[i];
      if let Some(free_term) = term.as_any_mut().downcast_mut::<FreeTerm>() {
        *free_variable_count += 1;
        if !available_terms.contains(free_term.as_ptr(), arg_eager) {
          free_term.compile_rhs_aliens(
            rhs_builder,
            variable_info,
            available_terms,
            arg_eager,
            max_arity,
            free_variable_count,
          );
        }
      } else {
        term.compile_rhs(rhs_builder, variable_info, available_terms, arg_eager);
      }
    }
  }

  fn scan_free_skeleton(
    &mut self,
    free_symbols : &mut Vec<FreeOccurrence>,
    other_symbols: &mut Vec<FreeOccurrence>,
    parent       : SlotIndex,
    arg_index    : ArgIndex,
  ) {
    let our_position = SlotIndex::from_usize(free_symbols.len());
    let occurrence   = FreeOccurrence::new(parent, arg_index, self.as_ptr());
    free_symbols.push(occurrence);

    for (i, t) in self.args.iter_mut().enumerate() {
      if let Some(f) = t.as_any_mut().downcast_mut::<FreeTerm>() {
        f.scan_free_skeleton(free_symbols, other_symbols, our_position, ArgIndex::from_usize(i));
      } else {
        let occurrence = FreeOccurrence::new(our_position, ArgIndex::from_usize(i), t.as_ptr());
        other_symbols.push(occurrence);
      }
    }
  }


  fn find_constraint_propagation_sequence(
    aliens        : &mut Vec<FreeOccurrence>,
    bound_uniquely: &mut NatSet,
    best_sequence : &mut ConstraintPropagationSequence,
  ) {
    let mut current_sequence: Vec<ArgIndex> = (0..aliens.len()).map(|i| ArgIndex::from_usize(i)).collect();
    best_sequence.cardinality = -1;

    Self::find_constraint_propagation_sequence_helper(
      aliens,
      &mut current_sequence,
      bound_uniquely,
      0,
      best_sequence
    );
    assert!(best_sequence.cardinality >= 0, "didn't find a sequence");
  }

  fn remaining_aliens_contain(
    aliens               : &Vec<FreeOccurrence>,
    current_sequence     : &Vec<ArgIndex>,
    step                 : usize,
    us                   : usize,
    interesting_variables: &NatSet,
  ) -> bool {
    if interesting_variables.is_empty() {
      return false;
    }
    for i in step..aliens.len() {
      if i != us && !interesting_variables.is_disjoint(aliens[current_sequence[i].idx()].term().occurs_below()) {
        return true;
      }
    }
    false
  }

  fn find_constraint_propagation_sequence_helper(
    aliens          : &mut Vec<FreeOccurrence>,
    current_sequence: &mut Vec<ArgIndex>,
    bound_uniquely  : &NatSet,
    mut step        : usize,
    best_sequence   : &mut ConstraintPropagationSequence,
  ) {
    let alien_count = aliens.len();

    // Add any alien that will "ground out match" to the current sequence.
    // By matching these early we maximize the chance of early match failure,
    // and avoid wasted work at match time.
    for i in step..alien_count {
      if aliens[current_sequence[i].idx()]
          .term()
          .will_ground_out_match(bound_uniquely)
      {
        current_sequence.swap(step, i);
        step += 1;
      }
    }
    if step < alien_count {
      // Now we search over possible ordering of remaining NGAs.

      let mut new_bounds: Vec<NatSet> = Vec::with_capacity(alien_count);
      // debug_advisory(&format!(
      //   "FreeTerm::findConstraintPropagationSequence(): phase 1 step = {}",
      //   step
      // ));

      for i in step..alien_count {
        new_bounds[i] = bound_uniquely.clone();
        let t = aliens[current_sequence[i].idx()].term_mut();
        t.analyse_constraint_propagation(&mut new_bounds[i]);

        // We now check if t has the potential to benefit from delayed matching.
        let unbound = t.occurs_below().difference(&new_bounds[i]);
        if !Self::remaining_aliens_contain(&aliens, &current_sequence, step, i, &unbound) {
          // No, so commit to matching it here.

          // debug_advisory(&format!(
          //   "FreeTerm::findConstraintPropagationSequence(): step = {} committed to {}",
          //   step, t
          // ));

          current_sequence.swap(step, i);
          Self::find_constraint_propagation_sequence_helper(
            aliens,
            current_sequence,
            &new_bounds[i],
            step + 1,
            best_sequence,
          );

          return;
        }
      }

      // We didn't find a NGA that we could commit to matching without possibly missing a better sequence.
      // Now go over the NGAs again. This time we need to consider expanding multiple branches in the
      // search tree.
      // debug_advisory(&format!(
      //   "FreeTerm::findConstraintPropagationSequence(): phase 2 step = {}",
      //   step
      // ));
      let mut expanded_at_least_one_branch = false;

      for i in step..alien_count {
        // We expand this branch if it binds something that could help another NGA.
        let newly_bound_uniquely: NatSet = new_bounds[i].difference(bound_uniquely);
        if Self::remaining_aliens_contain(&aliens, &current_sequence, step, i, &newly_bound_uniquely) {
          // Explore this path.

          // debug_advisory(&format!(
          //   "FreeTerm::findConstraintPropagationSequence(): step = {} exploring {}",
          //   step, aliens[current_sequence[i]].term()
          // ));
          current_sequence.swap(step, i);
          Self::find_constraint_propagation_sequence_helper(
            aliens,
            current_sequence,
            &new_bounds[i],
            step + 1,
            best_sequence,
          );
          current_sequence.swap(step, i);
          expanded_at_least_one_branch = true;
        }
      }
      if expanded_at_least_one_branch {
        return;
      }

      //	If we get here, none of the remaining NGAs can bind a variable that could affect
      //	the ability of other NGAs to bind variables, so there is no point pursuing further
      //	exploration. But we still need to union any other variable they may bind and score
      //	the result by making a recursive call to our leaf case.

      // debug_advisory(&format!(
      //   "FreeTerm::findConstraintPropagationSequence(): phase 3 step = {}",
      //   step
      // ));
      let mut new_bound_union = NatSet::new();
      for i in step..alien_count {
        new_bound_union.union_in_place(&new_bounds[i]);
      }

      Self::find_constraint_propagation_sequence_helper(
        aliens,
        current_sequence,
        &new_bound_union,
        alien_count,
        best_sequence,
      );
      return;
    }

    // Leaf of search tree.
    let n = bound_uniquely.len() as i32;
    if n > best_sequence.cardinality {
      best_sequence.sequence    = current_sequence.clone(); // deep copy
      best_sequence.bound       = bound_uniquely.clone();   // deep copy
      best_sequence.cardinality = n;
    }
  }

    /// Use the given automaton to compile this RHS. Maude's compileRhs3
    pub fn compile_into_automaton(
      &self,
      automaton      : &mut dyn RHSAutomaton,
      rhs_builder    : &mut RHSBuilder,
      variable_info  : &mut VariableInfo,
      available_terms: &mut TermBag,
      eager_context  : bool,
    ) -> VariableIndex {
      let arg_count = self.args.len();

      // We want to minimize conflict between slots to avoid quadratic number of
      // conflict arcs on giant right hand sides. The heuristic we use is crude:
      // we sort in order of arguments by number of symbol occurrences, and build
      // largest first.
      let mut order: Vec<(i32, usize)> = (0..arg_count)
          .map(|i| (-(self.args[i].compute_size() as i32), i))
          .collect();

      order.sort_unstable();

      // Consider each argument in largest first order.
      let symbol = self.symbol();
      let mut sources: Vec<VariableIndex> = vec![VariableIndex::Zero; arg_count];

      for (_, idx) in &order {
        let idx = *idx;
        let arg_is_eager = eager_context && symbol.eager_argument(idx);
        let mut term = self.args[idx].as_ptr();

        // Argument is free - see if we need to compile it into current automaton.
        if !available_terms.contains(term, arg_is_eager) {
          let free_term = term.as_any_mut()
                              .downcast_mut::<FreeTerm>()
                              .expect("term should be a FreeTerm");
          let source = free_term.compile_into_automaton(
            automaton,
            rhs_builder,
            variable_info,
            available_terms,
            arg_is_eager,
          );
          sources[idx] = source;
          term.core_mut().save_index = Some(source as VariableIndex);
          available_terms.insert_built_term(term, arg_is_eager);

          continue;
        }

        // Alien, variable or available term so use standard mechanism.
        sources[idx] = term.compile_rhs(rhs_builder, variable_info, available_terms, arg_is_eager);
      }

      // Need to flag last use of each source.
      for i in &sources {
        variable_info.use_index(*i);
      }

      // Add to free step to automaton.
      let index = variable_info.make_construction_index();
      if let Some(automaton) = automaton.as_any_mut().downcast_mut::<FreeRHSAutomaton>() {
        automaton.add_free(symbol.clone(), index, &sources);
      }

      index
    }
/*
    pub fn compile_remainder(&self, equation: RcPreEquation, slot_translation: &Vec<i32>) -> RcFreeRemainder {
      // Gather all symbols lying in or directly under free skeleton
      let mut free_symbols: Vec<FreeOccurrence> = Vec::new();
      let mut other_symbols: Vec<FreeOccurrence> = Vec::new();
      self.scan_free_skeleton(&mut free_symbols, &mut other_symbols, NONE, NONE);

      // Now classify occurrences of non Free-Theory symbols into 4 types
      let mut bound_variables: Vec<FreeOccurrence> = Vec::new(); // guaranteed bound when matched against
      let mut free_variables: Vec<FreeOccurrence> = Vec::new(); // guaranteed unbound when matched against
      let mut ground_aliens: Vec<FreeOccurrence> = Vec::new(); // ground alien subterms
      let mut non_ground_aliens: Vec<FreeOccurrence> = Vec::new(); // non-ground alien subterms

      let mut bound_uniquely = NatSet::new();

      for occ in &other_symbols {
        let t = occ.term();
        if let Some(v) = t.as_any().downcast_ref::<VariableTerm>() {
          let index = v.index as usize;
          if bound_uniquely.contains(index) {
            bound_variables.push(occ.clone());
          } else {
            bound_uniquely.insert(index);
            free_variables.push(occ.clone());
          }
        } else {
          if t.ground() {
            ground_aliens.push(occ.clone());
          } else {
            non_ground_aliens.push(occ.clone());
          }
        }
      }

      let mut best_sequence = ConstraintPropagationSequence::default();
      let mut sub_automata: Vec<RcLHSAutomaton> = Vec::new();
      let nr_aliens = non_ground_aliens.len();

      if nr_aliens > 0 {
        // Now we have to find a best sequence in which to match the
        // non-ground alien subterms and generate subautomata for them
        Self::find_constraint_propagation_sequence(&non_ground_aliens, &mut bound_uniquely, &mut best_sequence);

        for i in 0..nr_aliens {
          let (lhs_automata, _subproblem_likely) = non_ground_aliens[best_sequence.sequence[i] as usize]
              .term()
              .compile_lhs(false, &equation.borrow().variable_info, &mut bound_uniquely);
          sub_automata[i] = lhs_automata;
        }
        assert!(bound_uniquely == best_sequence.bound, "bound clash");
      }
      Rc::new(FreeRemainder::new(
        equation.clone(),
        &free_symbols,
        &free_variables,
        &bound_variables,
        &ground_aliens,
        &non_ground_aliens,
        &best_sequence.sequence,
        &sub_automata,
        slot_translation,
      ))
    }


    */
}

