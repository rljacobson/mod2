/*!

A `Module` owns all items defined within it. A module is a kind of namespace. Reduction/matching/evaluation
happens within the context of some module.

The `Module` structure is designed for Mod2 but can conceivably be used in other contexts. In any case, it implements 
algorithms any client application would also require.<br>

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

use mod2_abs::{
  HashMap,
  IString,
  warning,
  join_iter,
  heap_destroy,
};

use crate::{
  api::{
    symbol::SymbolPtr,
    term::BxTerm
  },
  core::{
    pre_equation::PreEquation,
    sort::{
      SortCollection,
      BxKind,
      Kind,
      KindError
    }
  },
};
#[cfg(feature = "profiling")]
use crate::core::profile::{StatementProfile, SymbolProfile};

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

  // ToDo: Q: Why not just have the sorts in `kinds`? Do we need `kinds` after construction?
  //       A: We use the symbol's "index within module" as a proxy for the symbol. (Not implemented.)
  pub sorts     : SortCollection,
  pub kinds     : Vec<BxKind>,
  pub symbols   : HashMap<IString, SymbolPtr>,
  pub variables : HashMap<IString, BxTerm>,
  pub equations : Vec<PreEquation>,
  pub rules     : Vec<PreEquation>,
  pub membership: Vec<PreEquation>,
  // pub strategies: Vec<PreEquation>, // Unimplemented

  // Members for performance profiling
  #[cfg(feature = "profiling")]
  symbol_info: Vec<SymbolProfile>,
  #[cfg(feature = "profiling")]
  mb_info    : Vec<StatementProfile>, // Membership
  #[cfg(feature = "profiling")]
  eq_info    : Vec<StatementProfile>, // Equation
  #[cfg(feature = "profiling")]
  rl_info    : Vec<StatementProfile>, // Rule
  #[cfg(feature = "profiling")]
  sd_info    : Vec<StatementProfile>, // Strategy Definition
}

impl Module {
  pub fn new(
    name      : IString,
    submodules: Vec<BxModule>,
    sorts     : SortCollection,
    symbols   : HashMap<IString, SymbolPtr>,
    variables : HashMap<IString, BxTerm>,
    equations : Vec<PreEquation>,
    rules     : Vec<PreEquation>,
    membership: Vec<PreEquation>,
  ) -> BxModule {
    Box::new(
      Module{
        name,
        status    : ModuleStatus::default(),
        submodules,
        kinds     : vec![], // computed below
        sorts,
        symbols,
        rules,
        equations,
        membership,
        variables,
  
        // Members for performance profiling
        #[cfg(feature = "profiling")]
        symbol_info: vec![],
        #[cfg(feature = "profiling")]
        mb_info    : vec![],
        #[cfg(feature = "profiling")]
        eq_info    : vec![],
        #[cfg(feature = "profiling")]
        rl_info    : vec![],
        #[cfg(feature = "profiling")]
        sd_info    : vec![],
      }
    )
  }
  
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
  pub fn compute_kind_closures(&mut self) {
    assert_eq!(self.status, ModuleStatus::Open, "tried to compute kind closure when module status is not open");
    // Temporarily swap out the sort collection with a dummy. 
    let mut sorts = SortCollection::new();
    std::mem::swap(&mut self.sorts, &mut sorts);

    for (_, sort) in
        sorts.iter()
             .filter(|(_, sort_ptr)| sort_ptr.kind.is_none())
    {
      let kind = Kind::new(sort);
      let kind = kind.unwrap_or_else(
        | kind_error | {
          // Maude sets the "is_bad" flag of a module in the case of a cycle in the Sort graph.
          let msg = kind_error.to_string();
          match kind_error {

            KindError::NoMaximalSort { kind, .. }
            | KindError::CycleDetected { kind, .. } => {
              warning!(1, "{}", msg.as_str());
              // Box::into_raw(kind)
              kind
            }

          }
        }
      );
      // The kind creates a maximal error sort as its first element that we have to add to the module.
      self.sorts.insert(kind.sorts[0]);

      // Maude sets the index_in_parent of the kind here.
      self.kinds.push(kind);
    }
    // Return the sort collection to self…
    std::mem::swap(&mut self.sorts, &mut sorts);
    // …and add the error sorts to the collection.
    self.sorts.append(sorts);
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
      let iter = self.symbols.iter().map(|(n, _)| n.as_ref());
      let sep = ", ";
      writeln!(
        f,
        "{}symbols: [{}]",
        inner_prefix,
        join_iter(iter, |_| sep).collect::<String>()
      )?;
    }
    //variables
    if !self.variables.is_empty() {
      // format_named_list(f, inner_prefix.as_str(), "variables", &self.variables)?
      let iter = self.variables.iter().map(|(name, _)| {name.to_string()});
      let sep = ", ";
      writeln!(
        f,
        "{}variables: [{}]",
        inner_prefix,
        join_iter(iter, |_| sep.to_string()).collect::<String>()
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
  /// A module owns its symbols and kinds, which are raw pointers to allocated memory. The module must reclaim this 
  /// owned memory when it is dropped.
  fn drop(&mut self) {
    for (_, symbol_ptr) in self.symbols.iter() {
      heap_destroy!(symbol_ptr.as_mut_ptr());
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


