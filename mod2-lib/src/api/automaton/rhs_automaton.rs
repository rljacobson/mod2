/*!

`RhsAutomaton` serves as the foundation for constructing and manipulating the right-hand side of
rewrite rules and equations during term rewriting. It provides a uniform interface for
building DAG nodes that represent the result of applying rewrite rules.

## Key Interface Methods

The class defines four essential pure virtual methods that concrete implementations must provide:

### Variable Index Management 
- *`remapIndices(VariableInfo& variableInfo)`* - Updates variable indices during compilation

### DAG Construction 

- *`construct(Substitution& matcher)`* - Creates new DAG nodes using variable bindings from pattern matching
- *`replace(DagNode* old, Substitution& matcher)`* - Performs in-place replacement of existing DAG nodes

### Stack Machine Support 
- *`recordInfo(StackMachineRhsCompiler& compiler)`* - Provides compilation support for the stack-based 
   interpreter, with a default implementation returning `false`

## Integration with Rewriting System

The class is extensively used throughout the Maude codebase. `RhsBuilder` manages collections of
`RhsAutomaton` instances, calling their methods during term construction. Terms compile themselves 
into `RhsAutomaton` instances through the `compileRhs` method.

*/

use crate::{
  core::{
    substitution::Substitution,
    VariableInfo,
  },
  api::dag_node::{DagNodePtr, MaybeDagNode},
};

pub type BxRHSAutomaton = Box<dyn RHSAutomaton>;

pub trait RHSAutomaton {
  fn as_any(&self) -> &dyn std::any::Any;
  fn as_any_mut(&mut self) -> &mut dyn std::any::Any;


  fn remap_indices(&mut self, variable_info: &mut VariableInfo);
  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode;
  fn replace(&mut self, old: DagNodePtr, matcher: &mut Substitution) -> DagNodePtr;

  // TODO: `StackMachineRhsCompiler` is not yet implemented.
  /*
  fn record_info(&self, _compiler: &mut StackMachineRhsCompiler) -> bool {
    false
  }
  */
}
