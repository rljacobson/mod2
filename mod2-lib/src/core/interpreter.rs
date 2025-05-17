/*!

In Maude, the interpreter is a global object that holds state about the current
execution. It holds the active module and execution settings. In particular, it
holds the configuration of the tracing system.

*/

use std::collections::HashSet;
use enumflags2::{bitflags, make_bitflags, BitFlags};
use mod2_abs::{IString, UnsafePtr};
use crate::core::BxModule;

pub type InterpreterPtr  = UnsafePtr<Interpreter>;
pub type ContinueFuncPtr = fn(&mut Interpreter, limit: usize, debug: bool);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum SearchKind {
  #[default]
  Search,
  Narrow,
  XGNarrow,
  SMTSearch,
  VUNarrow,
  FVUNarrow,
}

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum InterpreterAttribute {
  // Show (information) flags
  ShowCommand,
  ShowStats,
  ShowTiming,
  ShowBreakdown,

  // Loop mode flags
  ShowLoopStats,
  ShowLoopTiming,
  ERewriteLoopMode,

  // Memoization flags
  AutoClearMemo,

  // Profiler flags
  Profile,
  AutoClearProfile,

  // Debugger flags
  Break,

  // Tracer flags
  Trace,
  TraceCondition,
  TraceWhole,
  TraceSubstitution,
  TraceSelect,
  TraceMb,  // Membership
  TraceEq,  // Equation
  TraceRl,  // Rule
  TraceSd, // Sort
  TraceRewrite,
  TraceBody,
  TraceBuiltin,

  // Unimplemented Print attribute flags
  /*
  PrintAttribute,
  PrintAttributeNewline,
  */
  // Counter flags
  AutoClearRules,

  // Compiler flags
  CompileCount,
}

impl InterpreterAttribute {
  #![allow(non_upper_case_globals)]
  // Composite flags

  pub const ExceptionFlags: InterpreterAttributes =
    make_bitflags!(
    InterpreterAttribute::{
        Trace | Break | Profile
        // | PrintAttribute // Not implemented
      }
    );

  pub const DefaultFlags: InterpreterAttributes =
    make_bitflags!(
      InterpreterAttribute::{
        ShowCommand
        | ShowStats
        | ShowTiming
        | ShowLoopTiming
        | CompileCount
        | TraceCondition
        | TraceSubstitution
        | TraceMb
        | TraceEq
        | TraceRl
        | TraceSd
        | TraceRewrite
        | TraceBody
        | TraceBuiltin
        | AutoClearProfile
        | AutoClearRules
        // | PrintAttributeNewline
      }
    );
}

pub type InterpreterAttributes = BitFlags<InterpreterAttribute, u32>;

impl InterpreterAttribute {
  #[inline(always)]
  pub fn default() -> InterpreterAttributes {
    InterpreterAttribute::DefaultFlags
  }
}


#[derive()]
pub struct Interpreter {
  attributes    : InterpreterAttributes,
  current_module: Option<BxModule>,
  // print_flags   : PrintFlags,
  // current_view  : Option<SyntacticView>,

  // Continuation information
  // saved_state         : Option<CacheableState>,
  saved_solution_count: u64,                     // ToDo: As far as I know, this is nonnegative, so changed i64->u64.
  // saved_module        : Option<VisibleModule>,   // ToDo: Why is this a different type from `current_module`?
  continue_func       : Option<ContinueFuncPtr>,
  // saved_loop_subject  : Vec<Token>,              // ToDo: Why is the loop subject a syntactic structure?

  // ToDo: These objects are all referenced by _name_. Should they instead be referenced by index or something else?
  // ToDo: Could these `HashSet`s be `Vec`s?
  selected              : HashSet<IString>, // Temporary for building set of identifiers
  trace_names           : HashSet<IString>, // Names of symbols/labels selected for tracing
  pub(crate) break_names: HashSet<IString>, // Names of symbols/labels selected as break points
  excluded_modules      : HashSet<IString>, // Names of modules to be excluded from tracing
  concealed_symbols     : HashSet<IString>, // Names of symbols to have their arguments concealed during printing
}

impl Interpreter {
  pub fn attribute(&self, attribute: InterpreterAttribute) -> bool {
    self.attributes.contains(attribute)
  }

  pub fn trace_name(&self, name: &IString) -> bool {
    self.trace_names.contains(name)
  }

  pub fn excluded_module(&self, name: &IString) -> bool {
    self.excluded_modules.contains(name)
  }
}
