# In Progress

Determine the control flow of module initialization.

- Symbol info struct owned by module?
- Module initialization
  - symbol fixups (type checking identities)

# Next Steps

- compiler
   - `Term::compile_lhs`, `Term::compile_rhs`
   - Implement `FreeTerm::compile_lhs`
   - uncomment `api/free_theory/compiler.rs`
   - `core::automata::*` - uncomment `binding_lhs_automaton`, `copy_rhs_automaton`, and `trivial_rhs_automata`.
  - `StackMachineRhsCompiler`

- rewriter
  - rewriting methods on `core::rewriting_context::context::RewritingContext`
- Incorporate the remaining commented automata in the free theory: 
  * FreeFast3RHSAutomaton
  * FreeFast2RHSAutomaton
  * FreeTernaryRHSAutomaton
  * FreeBinaryRHSAutomaton
  * FreeUnaryRHSAutomaton
  * FreeNullaryRHSAutomaton

# Saved for later

- `AUExtensionInfo`, `AUCExtensionInfo`, `SExtensionInfo`
- `core::rewriting_context::trace`
- `core::rewriting_context::debugger`
- `FreeSymbol::complex_strategy` - execution strategies are not implemented.

- Tracing in:
    - `PreEquation::check_condition_find_first`
    - `FreeRemainder`
    - `SortConstraintTable`
    - `MemoMap`
    - `SymbolCore`

