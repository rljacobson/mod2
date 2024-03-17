/*!

A `Module` owns all items defined within it. A module is a kind of namespace. Reduction/matching/evaluation
happens within the context of some module.<br>

## Module Construction

The initialization of a module involves several steps which is tracked by the `ModuleStatus` enum. I've included the
same statuses as Maude, but it's not clear to me if I'll need them.

### Closure of the Sort Set

The connected components of the lattice of sorts (the "kinds") is computed by computing the transitive closure of the
subsort relation.

*/

use crate::{
  abstractions::{
    HashMap
    ,
    IString
  },
  theory::symbol::RcSymbol
};
use crate::core::sort::collection::SortCollection;
use crate::core::sort::kind::RcKind;

#[derive(Copy, Clone, Default)]
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

  // Sorts
  pub sorts     : SortCollection, // ToDo: Why not just have the sorts in `kinds`?
  pub kinds     : Vec<RcKind>,
  pub symbols   : HashMap<IString, RcSymbol>,
  // pub sort_constraints: Vec<RcPreEquation>,
  // pub equations       : Vec<RcPreEquation>,
  // pub rules           : Vec<RcPreEquation>,

  // ProfileModule members (performance profiling)
  // symbol_info: Vec<SymbolProfile>,
  // mb_info    : Vec<StatementProfile>, // Membership
  // eq_info    : Vec<StatementProfile>, // Equation
  // rl_info    : Vec<StatementProfile>, // Rule
  // sd_info    : Vec<StatementProfile>, // Strategy Definition
}

impl Module {
  pub fn new(name: IString) -> Module {
    Module {
      name,
      ..Module::default()
    }
  }

  /// Computes the transitive closure of the connected components of the lattice of sorts. Each connected component is called a kind.
  pub fn compute_kind_closures(&mut self) {

  }

}
