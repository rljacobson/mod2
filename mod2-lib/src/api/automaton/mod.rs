/*!

The matcher automatons.

*/

mod lhs_automaton;
mod rhs_automaton;

pub use lhs_automaton::{BxLHSAutomaton, LHSAutomaton};
pub use rhs_automaton::{BxRHSAutomaton, RHSAutomaton};

/*
///	This trait must be derived from for equational theories that generate
///	unification subproblems.
pub(crate) trait UnificationSubproblem {}

//	These traits should be implemented for theories supported by the stack-based interpreter.
pub(crate) trait Instruction {}
/// instruction with regular GC handling
pub(crate) trait RegularInstruction {}
/// regular instruction that is not the last instruction in its sequence
pub(crate) trait NonFinalInstruction {}
/// regular ctor that is not the last instruction in its sequence
pub(crate) trait NonFinalConstructor {}
/// regular extor that is not the last instruction in its sequence
pub(crate) trait NonFinalExecutor {}
/// regular instruction that is the final instruction in its sequence
pub(crate) trait FinalInstruction {}
*/
