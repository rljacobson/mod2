/*!
An execution strategy determines the order and manner in which function arguments are
evaluated. Evaluation of arguments is either eager, meaning arguments are fully evaluated
before the operator is applied, or lazy, meaning evaluation is deferred until needed.
*/

use mod2_abs::NatSet;

/// The execution strategy.
#[derive(Default, Eq, PartialEq)]
pub struct Strategy {
  /// This flag serves as an optimization hint for the Maude rewriting system.
  /// When `unevaluated_arguments` is true, it indicates that some arguments to
  /// operators using this strategy will not be evaluated during rewriting, which
  /// affects how the system handles argument processing and reduction strategies.
  pub unevaluated_arguments: bool,
  /// The normalized evaluation sequence
  pub strategy             : Vec<u16>,
  /// Set of argument positions that should be evaluated eagerly
  pub eager                : NatSet,
  /// Set of argument positions that get evaluated at all
  pub evaluated            : NatSet,
  /// Set of argument positions that should never be evaluated
  pub frozen               : NatSet,
}

impl Strategy {
  pub fn new() -> Box<Self> {
    Box::new(Strategy::default())
  }
}