/*!

Holds state for a rewrite system.

Maude uses inheritance: A `RewritingContext` is-a `Substitution`. We use composition: A `RewritingContext` has-a
`Substitution`. Maude source says:

> A rewriting context keeps track of miscellaneous information needed while rewriting. An important performance trick is
> that we derive it from Substitution so that we can use the rewriting context to construct matching substitutions in.
> This avoids creating a new substitution at the start of each match attempt.

I interpret this to mean that the Substitution data structure is reused between matches instead of created and
destroyed.

ToDo: This implements way more of Maude than a pattern matching library should have. Refactor to remove
      application-specific infrastructure.

*/

// pub mod debugger;
// pub mod trace;
mod context;

use std::fmt::{Display, Formatter};
use enumflags2::{bitflags, BitFlags};

pub use context::{RewritingContext, BxRewritingContext};

#[bitflags]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u16)]
pub enum ContextAttribute {
  LocalTrace,
  Trace,
  TracePost,
  Abort,
  Info,
  CtrlC,
  Step,
  Interactive,
  Silent,
  DebugMode,
}
pub type ContextAttributes = BitFlags<ContextAttribute, u16>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Purpose {
  ConditionEval,
  SortEval,
  TopLevelEval,
  MetaEval,
  Other,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum RewriteType {
  Normal,
  Builtin,
  Memoized,
}

impl Display for RewriteType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      RewriteType::Normal => {
        write!(f, "normal")
      }
      RewriteType::Builtin => {
        write!(f, "built-in")
      }
      RewriteType::Memoized => {
        write!(f, "memoized")
      }
    }
  }
}
