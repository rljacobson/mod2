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

  // ProfileModule members (performance profiling)
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

}


impl Drop for Module {
  fn drop(&mut self) {
    for (_, symbol_ptr) in self.symbols.iter() {
      unsafe {
        heap_destroy!(*symbol_ptr);
      }
    }
  }
}
