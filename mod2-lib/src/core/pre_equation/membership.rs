/*!

Methods that are specific to sort constraints (membership axioms) can be called like this:

```ignore
membership::fast_variable_count(&this);
```

*/
use nu_ansi_term::Color::Magenta;
use mod2_abs::{
  tracing::warn,
  IString
};
use crate::{
  api::term::BxTerm,
  core::{
    format::{
      FormatStyle,
      Formattable
    },
    pre_equation::{
      condition::Conditions,
      PreEquation,
      PreEquationAttribute,
      PreEquationKind
    },
    sort::SortPtr
  },
  UNDEFINED,
};
use crate::core::VariableIndex;

pub fn new(name: Option<IString>, lhs_term: BxTerm, sort: SortPtr, condition: Conditions) -> PreEquation {
  PreEquation {
    name,
    attributes: Default::default(),

    pe_kind: PreEquationKind::Membership {
      sort
    },
    conditions   : condition,
    lhs_term,
    lhs_automaton: None,
    lhs_dag      : None,
    variable_info: Default::default(),

    index_within_parent_module: UNDEFINED,
  }
}

pub(crate) fn check(this: &mut PreEquation) {
  if !this.is_nonexec() && !this.variable_info.unbound_variables.is_empty() {
    let mindex = this.variable_info.unbound_variables.min_value().unwrap();
    let min_variable = this.variable_info.index_to_variable(mindex as VariableIndex).unwrap();
    let mut this_repr = String::new();
    this.repr(&mut this_repr, FormatStyle::Simple).unwrap();

    // ToDo: Figure out reporting of language errors.
    let warning = format!(
      "{}: variable {} is used before it is bound in {}:\n{}",
      Magenta.paint(&this_repr),
      min_variable,
      this.pe_kind.noun(),
      this_repr
    );
    warn!(warning);

    // No legitimate use for such sort constraints so mark it as bad.
    this.attributes |= PreEquationAttribute::Bad;
  }
}

/*
pub fn compile(this: &mut PreEquation, compile_lhs: bool) {
  if this.attributes.contains(PreEquationAttribute::Compiled) {
    return;
  }
  this.attributes.insert(PreEquationAttribute::Compiled);
  let mut available_terms = TermBag::default(); // terms available for reuse
  this.compile_build(&mut available_terms, false);
  this.compile_match(compile_lhs, false);
}
*/
