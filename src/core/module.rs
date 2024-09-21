/*!

A `Module` owns all items defined within it. A module is a kind of namespace. Reduction/matching/evaluation
happens within the context of some module.<br>

## Module Construction

The initialization of a module involves several steps which is tracked by the `ModuleStatus` enum. I've included the
same statuses as Maude, but it's not clear to me if I'll need them.

### Closure of the Sort Set

The connected components of the lattice of sorts (the "kinds") is computed by computing the transitive closure of the
subsort relation. This is done by calling the method `Module::compute_kind_closures(…)`.

## See Also...

 * The module system section of the [Design Notes](crate).

*/

use std::fmt::{Debug, Display, Formatter};
use tiny_logger::{Channel, log};
use crate::{
  abstractions::{
    HashMap,
    IString,
    Channel,
    log
  },
  core::{
    sort::{
      collection::SortCollection,
      kind::{
        Kind,
        BxKind,
        KindPtr
      },
      kind_error::KindError,
    },
    pre_equation::PreEquation,
  },
  heap_destroy,
  theory::symbol::{
    Symbol,
    SymbolPtr
  },
};
use crate::abstractions::join_iter;
use crate::core::sort::kind::{BxKind, KindPtr};
use crate::core::sort::Sort;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Debug)]
pub enum ModuleStatus {
  #[default]
  Open,
  SortSetClosed,
  SignatureClosed,
  FixUpsClosed,
  TheoryClosed,
  StackMachineCompiled,
}

pub type BxModule = Box<Module>;

#[derive(Default)]
pub struct Module {
  pub name      : IString,
  pub submodules: Vec<BxModule>,
  pub status    : ModuleStatus,

  // ToDo: Why not just have the sorts in `kinds`? Do we need `kinds` after construction?
  pub sorts     : SortCollection,
  pub kinds     : Vec<BxKind>,
  pub symbols   : HashMap<IString, SymbolPtr>,
  pub equations : Vec<PreEquation>,
  pub rules     : Vec<PreEquation>,
  pub membership: Vec<PreEquation>,
  // pub strategies: Vec<PreEquation>, // Unimplemented

  // Members for performance profiling
  // symbol_info: Vec<SymbolProfile>,
  // mb_info    : Vec<StatementProfile>, // Membership
  // eq_info    : Vec<StatementProfile>, // Equation
  // rl_info    : Vec<StatementProfile>, // Rule
  // sd_info    : Vec<StatementProfile>, // Strategy Definition
}

impl Module {
  /**
  Computes the transitive closure of the subsort relation, constructing the lattice of sorts. This only needs to be
  done once when the module is constructed. It is not idempotent.

  The `ModuleAST::construct(…)` method calls this method automatically, so any module constructed by the parser,
  for example, will not need to have this method called on it.

  Before this method call, a module will have `status == ModuleStatus::Open`. The method sets the status to
  `ModuleStatus::SortSetClosed`, so at any point after this method call, a module will have
  `status >= ModuleStatus::SortSetClosed`.

  ToDo: It would be nice if this method were idempotent. Low priority.
  */
  pub unsafe fn compute_kind_closures(&mut self) {
    assert_eq!(self.status, ModuleStatus::Open, "tried to compute kind closure when module status is not open");

    for (_, sort) in
        self.sorts
            .iter()
            .filter(|(_, sort_ptr)| (**sort_ptr).kind.is_null())
    {
      let kind = unsafe { Kind::new(sort) };
      let mut kind = kind.unwrap_or_else(
        | kind_error | {
          // Maude sets the "is_bad" flag of a module in the case of a cycle in the Sort graph.
          let msg = kind_error.to_string();
          match kind_error {

            KindError::NoMaximalSort { kind, .. }
            | KindError::CycleDetected { kind, .. } => {
              log(Channel::Warning, 1, msg.as_str());
              // Box::into_raw(kind)
              kind
            }

          }
        }
      );

      // Maude sets the index_in_parent of the kind here.
      self.kinds.push(kind);
    }
    self.status = ModuleStatus::SortSetClosed
  }


  /// Formats the module for display with `prefix` for each line. The `Debug` impl defers to this method. Interior
  /// indentation is affixed to `prefix`.
  fn debug_fmt(&self, f: &mut Formatter<'_>, prefix: &String) -> std::fmt::Result {
    let inner_prefix = format!("{}{}", prefix, " ".repeat(crate::DISPLAY_INDENT));
    writeln!(f, "{}Module {{", prefix)?;
    writeln!(f, "{}name: {}", inner_prefix, self.name)?;
    writeln!(f, "{}status: {:?}", inner_prefix, self.status)?;
    //sorts (as kinds)
    if !self.kinds.is_empty()  {
      format_named_list(f, inner_prefix.as_str(), "sorts", &self.kinds)?
      // let sort_vec = join_iter(self.sorts.iter().map(|(name, _)| name.as_str()), |_| ", ",).collect::<String>();
      // writeln!(f, "{}sorts: [{}]", inner_prefix, sort_vec)?;
    }
    //symbols
    if !self.symbols.is_empty() {
      let iter = self.symbols.iter().map(|(n, _)| n.as_str());
      let sep = ", ";
      writeln!(
        f,
        "{}symbols: [{}]",
        inner_prefix,
        join_iter(iter, |_| sep).collect::<String>()
      )?;
    }
    //equations
    if !self.equations.is_empty() {
      format_named_list(f, inner_prefix.as_str(), "equations", &self.equations)?
    }
    //rules
    if !self.rules.is_empty() {
      format_named_list(f, inner_prefix.as_str(), "rules", &self.rules)?
    }
    //membership
    if !self.membership.is_empty() {
      format_named_list(f, inner_prefix.as_str(), "membership", &self.membership)?
    }
    //modules
    for module in &self.submodules {
      module.debug_fmt(f, &inner_prefix)?;
    }
    writeln!(f, "{}}}", prefix)

  }

}

impl Drop for Module {
  /// A module owns its symbols, which are raw pointers to allocated memory. The module must reclaim this owned memory
  /// when it is dropped.
  fn drop(&mut self) {
    for (_, symbol_ptr) in self.symbols.iter() {
      unsafe {
        heap_destroy!(*symbol_ptr);
      }
    }
  }
}

impl Debug for Module {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let prefix = "".to_string();
    self.debug_fmt(f, &prefix)
  }
}


/// Helper function to format a named list of something:
/// ```txt
/// thing_name: [
///   thing1
///   thing2
///   thing3
/// ]
/// ```
fn format_named_list<T: Display>(f: &mut Formatter<'_>, prefix: &str, name: &str, list: &Vec<T>)
  -> std::fmt::Result
{
  let tab = " ".repeat(crate::DISPLAY_INDENT);
  writeln!(f, "{}{}: [", prefix, name)?;
  for item in list.iter() {
    writeln!(f, "{}{}{}", prefix, tab, item)?;
  }
  writeln!(f, "{}]", prefix)
}



#[cfg(test)]
mod tests {
  use std::assert_matches::assert_matches;
  use lalrpop_util::{
    lexer::Token,
    ParseError
  };
  use crate::parser::ast::ModuleAST;
  use super::*;

  fn parse_ex1() -> Result<Box<ModuleAST>, ()>{
    let path = "examples/example1.mod2";
    let text = match std::fs::read_to_string(path) {
      Ok(s) => { s }
      Err(e) => {
        panic!("Failed to read {}: {}", path, e);
      }
    };

    let parser = crate::parser::parser::ModuleParser::new();
    let result: Result<Box<ModuleAST>, ParseError<usize, Token, &str>> =  parser.parse(&text);
    match result {
      Ok(ast) => {
        println!("SUCCESS!");
        return Ok(ast);
      },
      Err(e) => {
        eprintln!("Parse error: {}", e);
        return Err(());
      }
    }
  }


  #[test]
  fn test_ex1_construction() {
    let ast: Box<ModuleAST> =  parse_ex1().expect("Failed to parse module");
    let constructed = ast.construct_module();
    println!("{:?}", constructed);
  }
}
