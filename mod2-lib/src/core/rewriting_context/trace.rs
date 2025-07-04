use std::sync::atomic::{AtomicBool, Ordering};
use mod2_abs::{
  log::debug,
  NatSet
};
use super::{ContextAttribute, RewriteType};
use crate::{
  api::{DagNode, DagNodePtr},
  core::{
    substitution::{
      print_substitution_with_ignored,
      MaybeDagNode,
      print_substitution_narrowing,
      print_substitution,
      Substitution
    },
    pre_equation::{PreEquationKind::*, PreEquation},
    format::{Formattable, FormatStyle},
    rewriting_context::context::RewritingContext,
    interpreter::{InterpreterAttribute, InterpreterPtr},
    NarrowingVariableInfo,
    Module,
  }
};

#[derive(Copy, Clone)]
pub struct VariantTraceInfo<'a> {
  old_variant_substitution: &'a Substitution,
  new_variant_substitution: &'a Substitution,
  original_variables:       &'a NarrowingVariableInfo,
}

/// Tracing status is global for all `RewritingContext`s.
static TRACE_STATUS: AtomicBool = AtomicBool::new(false);

// ToDo: Make trace_status local to context or interpreter
/// Tracing status is global for all `RewritingContext`s.
pub fn trace_status() -> bool {
  TRACE_STATUS.load(Ordering::Relaxed)
}
/// Tracing status is global for all `RewritingContext`s.
pub fn set_trace_status(status: bool) {
  TRACE_STATUS.store(status, Ordering::Relaxed);
}


impl RewritingContext {
  pub fn do_not_trace(&self, redex: DagNodePtr, pe: Option<&PreEquation>) -> bool {
    let symbol = redex.borrow().symbol();
    let interpreter = self.interpreter;
    (interpreter.attribute(InterpreterAttribute::TraceSelect)
      && !(interpreter.trace_name(&symbol.name())
        || (pe.is_some() && pe.unwrap().name.is_some() && interpreter.trace_name(&pe.unwrap().name.unwrap()))))
      || (pe.is_none() && !interpreter.attribute(InterpreterAttribute::TraceBuiltin))
      || interpreter.excluded_module(&symbol.get_module().upgrade().unwrap().borrow().name)
  }

  /* Print attributes are unimplemented.
  pub fn check_for_print_attribute(
    &self,
    item_type: ItemType,
    item: Option<&PreEquation>,
  ) {
    if let Some(item) = item {
      let module = item.get_module();
      if let Some(pa) = module.print_attribute(item_type, item) {
        pa.print(io::stdout()).unwrap();
        if Interpreter::attribute(InterpreterAttribute::PrintAttribute_NEWLINE) {
          println!();
        }
      }
    }
  }
  */

  pub fn trace_pre_eq_application(
    &mut self,
    redex: MaybeDagNode,
    maybe_equation: Option<&PreEquation>,
    eq_type: RewriteType,
  ) {
    // All unusual situations during an equational rewrite are funneled
    // through this function, by setting the traceFlag in class Module.
    // This is so that rewriting only has to test a single flag
    // to get off the fast case, and into the (slow) exception case.
    //
    // Possible unusual situations:
    // (1) Profiling is enabled
    // (2) Statement print attributes are enabled
    // (3) Aborting the computation
    // (4) Single stepping in debugger
    // (5) Breakpoints are enabled
    // (6) ^C interrupt
    // (7) Info interrupt
    // (8) Tracing is enabled

    if redex.is_none() {
      // Only relevant for the Rule case.
      // Dummy rewrite; need to ignore the following trace_post_rule_rewrite() call.
      self.attributes.remove(ContextAttribute::TracePost);
      return;
    }

    // TODO: Handle the `equation==None` case
    let equation = maybe_equation.unwrap();

    let redex: DagNodePtr = redex.unwrap();
    let redex_ref: &dyn DagNode = &*redex.as_ref();
    let interpreter = self.interpreter;

    if interpreter.attribute(InterpreterAttribute::Profile) {
      // Todo: Is `self.root` gauranteed to exist?
      let mut profile_module = self
        .root
        .unwrap()
        .node()
        .symbol()
        .get_module()
        .upgrade()
        .unwrap()
        .borrow_mut();
      // TODO: Unify `profile_*_rewrite` code
      profile_module.profile_eq_rewrite(redex.clone(), Some(equation), eq_type);
    }
    // Print attributes are not implemented
    // if interpreter.attribute(InterpreterAttribute::PrintAttribute) {
    //   self.check_for_print_attribute(MetadataStore::EQUATION, equation);
    // }

    // handeDebug() takes care of abort, single stepping, breakpoints,
    // ^C interrupts and info interrupts. These are common to
    // all rewrite types.
    let attribute = ContextAttribute::LocalTrace;
    if self.handle_debug(redex.clone(), Some(equation))
      || !self.attributes.contains(attribute)
      || !interpreter.attribute(InterpreterAttribute::TraceEq)
      || self.do_not_trace(redex.clone(), Some(equation))
    {
      self.attributes.remove(ContextAttribute::TracePost);
      return;
    }
    self.attributes.set(ContextAttribute::TracePost);

    if interpreter.attribute(InterpreterAttribute::TraceBody) {
      println!("{} {}", HEADER, equation.kind.noun());
    }

    // TODO: Fix whenever we figure out the `equation==None` case
    if let Some(equation) = maybe_equation {
      if interpreter.attribute(InterpreterAttribute::TraceBody) {
        println!("{}", equation.repr(FormatStyle::Default));

        if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
          print_substitution_with_ignored(&self.substitution, &equation.variable_info, &NatSet::default());
        }
      } else {
        if let Some(label) = &equation.name {
          println!("{}", label);
        } else {
          println!("(unlabeled {})", equation.kind.noun());
        }
      }
    } else {
      // equation == None case
      // TODO: We still need the noun!
      if eq_type != RewriteType::Normal {
        println!(
          "({} {} for symbol {})",
          eq_type,
          equation.kind.noun(),
          redex_ref.symbol().repr(FormatStyle::Simple)
        );
      }
    }

    match &equation.kind {
      StrategyDefinition { .. } => {
        unimplemented!("Strategy language is not implemented.")
      }

      SortConstraint { sort, .. } => {
        if interpreter.attribute(InterpreterAttribute::TraceWhole) {
          if let Some(root) = &self.root {
            println!("Whole: {}", root.borrow());
          } else {
            println!("Whole: <root is None>");
          }
        }
        if interpreter.attribute(InterpreterAttribute::TraceRewrite) {
          // TODO: We are assuming the redex does have a sort. Check that this is guaranteed for sort constraints.
          println!(
            "{}: {} becomes {}",
            redex_ref.get_sort().unwrap().borrow(),
            redex_ref,
            sort.borrow()
          );
        }
      }

      _ => {
        if interpreter.attribute(InterpreterAttribute::TraceWhole) {
          if let Some(root) = &self.root {
            println!("Old: {}", root.borrow());
          } else {
            println!("Old: <root is None>");
          }
        }
        if interpreter.attribute(InterpreterAttribute::TraceRewrite) {
          println!("{} \n--->", redex_ref);
        }
      }
    }

    debug!(1, redex_ref.to_string().as_str());
  }

  pub fn trace_post_eq_application(&self, replacement: DagNodePtr) {
    let attribute = ContextAttribute::TracePost;
    if self.attributes.contains(attribute) {
      let attribute = ContextAttribute::Abort;
      assert!(!self.attributes.contains(attribute), "abort flag set");
      let interpreter = self.interpreter;

      if interpreter.attribute(InterpreterAttribute::TraceRewrite) {
        println!("{}", replacement.borrow().to_string());
      }

      debug!(1, replacement.borrow().to_string().as_str());

      if interpreter.attribute(InterpreterAttribute::TraceWhole) {
        if let Some(root) = &self.root {
          println!("New: {}", root.borrow());
        } else {
          println!("New: <root is None>");
        }
      }
    }
  }

  pub fn trace_narrowing_step(
    &mut self,
    pre_equation: &PreEquation,
    redex: DagNodePtr,
    replacement: DagNodePtr,
    variable_info: &NarrowingVariableInfo,
    substitution: &Substitution,
    new_state: DagNodePtr,
    variant: Option<VariantTraceInfo<'_>>, // None for Rule, Some for Equation
  ) {
    let interpreter = self.interpreter;
    let attribute = ContextAttribute::LocalTrace;
    if self.handle_debug(redex.clone(), Some(pre_equation))
      || !self.attributes.contains(attribute)
      || !interpreter.attribute(InterpreterAttribute::TraceRl)
      || self.do_not_trace(redex.clone(), Some(pre_equation))
    {
      return;
    }

    if interpreter.attribute(InterpreterAttribute::TraceBody) {
      if variant.is_some() {
        println!(" {}", Paint::cyan("variant narrowing step"));
      } else {
        println!("{}", Paint::magenta("narrowing step"));
      }
      println!("{}", pre_equation.repr(FormatStyle::Simple));

      if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
        if variant.is_some() {
          println!("Equation variable bindings:");
          print_substitution_with_ignored(substitution, &pre_equation.variable_info, &NatSet::default());
          println!("Old variant variable bindings:");
        } else {
          println!("Rule variable bindings:");
          print_substitution_with_ignored(substitution, &pre_equation.variable_info, &NatSet::default());
          println!("Subject variable bindings:");
        }

        let subject_variable_count = variable_info.variable_count();
        if subject_variable_count == 0 {
          println!("empty substitution");
        } else {
          // TODO: Is it guaranteed that pre_equation has a module?
          let variable_base = pre_equation
            .get_module()
            .upgrade()
            .unwrap()
            .borrow()
            .minimum_substitution_size;
          for i in 0..subject_variable_count {
            let v = variable_info.index_to_variable(i);
            let d = substitution.value(variable_base as usize + i);

            assert!(v.is_some(), "null variable");
            print!("{} --> ", v.unwrap().borrow());

            if let Some(d) = d {
              println!("{}", d.borrow());
            } else {
              println!("(unbound)");
            }
          }
        }
      }
    }

    if interpreter.attribute(InterpreterAttribute::TraceWhole) {
      if let Some(VariantTraceInfo {
        old_variant_substitution,
        original_variables,
        ..
      }) = variant
      {
        if let Some(root) = &self.root {
          println!("\nOld variant: {}", root.node());
        } else {
          println!("\nOld variant: <root is None>");
        }
        print_substitution_narrowing(&old_variant_substitution, original_variables);
        println!();
      } else {
        if let Some(root) = &self.root {
          println!("\nOld: {}", root.node());
        } else {
          println!("\nOld: <root is None>");
        }
      }
    }

    if interpreter.attribute(InterpreterAttribute::TraceRewrite) {
      println!("{} \n--->\n{}", redex, replacement);
    }

    if interpreter.attribute(InterpreterAttribute::TraceWhole) {
      if let Some(VariantTraceInfo {
        new_variant_substitution,
        original_variables,
        ..
      }) = variant
      {
        println!("\nNew variant: {}", new_state);
        print_substitution_narrowing(&new_variant_substitution, original_variables);
        println!();
      } else {
        println!("New: {}", new_state);
      }
    }
  }

  /* Strategy language not implemented
    pub fn trace_strategy_call(
      &self,
      sdef: &StrategyDefinition,
      call_dag: DagNodePtr,
      subject: DagNodePtr,
      substitution: &Substitution,
    ) {
      let interpreter = self.interpreter;
      if interpreter.attribute(InterpreterAttribute::Profile) {
        if let Some(profile_module)
            = self.root
            .borrow()
            .symbol()
            .get_module()
            .upgrade()
            .unwrap()
            .borrow() {
          profile_module.profile_sd_rewrite(subject, sdef);
        }
      }
      // if interpreter.attribute(InterpreterAttribute::PrintAttribute) {
      //   check_for_print_attribute(MetadataStore::STRAT_DEF, sdef);
      // }

      if self.handle_debug(call_dag.clone(), sdef)
          || !self.local_trace_flag
          || !interpreter.attribute(InterpreterAttribute::TraceSd)
          || self.do_not_trace(call_dag.clone(), Some(sdef))
      {
        return;
      }

      if interpreter.attribute(InterpreterAttribute::TraceBody) {
        println!("{} strategy call", HEADER);
        println!("{}", sdef);
        // call_dags uses the auxiliary symbol we should print it readable
        let call_dag = call_dag.borrow();
        if call_dag.symbol().arity() > 0 {
          println!(
            "call term --> {}({})",
            Token::name(sdef.get_strategy().name()),
            join_iter(call_dag.iter_args(), |_| ", ").collect::<String>()
          );
        }

        if interpreter.attribute(InterpreterAttribute::TraceWhole) {
          println!("subject --> {}", subject.borrow());
        }

        if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
          print_substitution_narrowing(substitution, sdef.variable_info);
        }

      } else {
        if let Some(label) = sdef.name(){
          println!("{}", label);
        } else {
          let strat_id = sdef.get_strategy().name();
          println!("{} (unlabeled definition)", strat_id);
        }
      }
    }
  */


  pub(crate) fn trace_begin_trial(&mut self, subject: DagNodePtr, pre_equation: &PreEquation) -> Option<i32> {
    // assert!(equation != 0, "null equation in trial");

    let interpreter: InterpreterPtr = self.interpreter;

    if interpreter.attribute(InterpreterAttribute::Profile) {
      let module: &mut Module = &mut *self
        .root
        .unwrap()
        .node()
        .symbol()
        .get_module()
        .upgrade()
        .unwrap()
        .borrow_mut();
      module.profile_condition_start(pre_equation);
    }

    if self.handle_debug(subject.clone(), Some(pre_equation)) {
      return None;
    }

    let attribute = ContextAttribute::LocalTrace;
    if !self.attributes.contains(attribute)
      || !interpreter.attribute(pre_equation.kind.interpreter_trace_attribute())
      || self.do_not_trace(subject.clone(), Some(pre_equation))
    {
      return None;
    }

    println!(
      "{}trial #{}\n{}",
      HEADER,
      self.trial_count + 1,
      pre_equation.repr(FormatStyle::Default)
    );
    if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
      print_substitution_with_ignored(&self.substitution, &pre_equation.variable_info, &NatSet::new());
    }

    Some((self.trial_count + 1) as i32)
  }

  pub(crate) fn trace_end_trial(&self, trial_ref: Option<i32>, success: bool) {
    let attribute = ContextAttribute::Abort;
    if !self.attributes.contains(attribute) && trial_ref.is_some() {
      println!(
        "{}{} #{}",
        HEADER,
        if success {
          "success"
        } else {
          "failure"
        },
        trial_ref.unwrap()
      );
    }
  }

  pub(crate) fn trace_exhausted(&self, trial_ref: Option<i32>) {
    let attribute = ContextAttribute::Abort;
    if !self.attributes.contains(attribute) && trial_ref.is_some() {
      println!("{}exhausted (#{})", HEADER, trial_ref.unwrap());
    }
  }

  pub(crate) fn trace_begin_fragment(&self, trial_ref: Option<i32>, fragment: &ConditionFragment, first_attempt: bool) {
    let attribute = ContextAttribute::Abort;
    if self.attributes.contains(attribute) || trial_ref.is_none() {
      return;
    }
    // let fragment: Ref<ConditionFragment> = pre_equation.condition()[fragment_index].borrow();
    let prefix = if first_attempt { "" } else { "re-" };
    println!(
      "{}{}solving condition fragment\n{}",
      HEADER,
      prefix,
      fragment.repr(FormatStyle::Simple)
    );
  }

  pub(crate) fn trace_end_fragment(
    &self,
    trial_ref: Option<i32>,
    pre_equation: &PreEquation,
    fragment_index: usize,
    success: bool,
  ) {
    if self
      .interpreter
      .upgrade()
      .unwrap()
      .attribute(InterpreterAttribute::Profile)
    {
      if let Some(module) = pre_equation.parent_module.upgrade() {
        module.borrow().profile_fragment(pre_equation, fragment_index, success);
      }
    }

    let attribute = ContextAttribute::Abort;
    if self.attributes.contains(attribute) || trial_ref.is_none() {
      return;
    }

    let fragment = &pre_equation.condition()[fragment_index].borrow();
    if success {
      println!(
        "{}success for condition fragment\n{}",
        HEADER,
        fragment.repr(FormatStyle::Simple)
      );
      if self
        .interpreter
        .upgrade()
        .unwrap()
        .attribute(InterpreterAttribute::TraceSubstitution)
      {
        print_substitution(&self.substitution, &pre_equation.variable_info);
      }
    } else {
      println!(
        "{}failure for condition fragment\n{}",
        HEADER,
        fragment.repr(FormatStyle::Simple)
      );
    }
  }
}
