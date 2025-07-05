# In Progress

- `SortIndex`, `SlotIndex`, `VariableIndex`
- `FreeSymbol::discrimination_net.apply_replace(subject, context)`
- `FreeSymbol::complex_strategy`

# Next Steps

- `PreEquation::check_condition_find_first`

- compiler
   - `Term::compile_lhs`, `Term::compile_rhs`
   - Implement `FreeTerm::compile_lhs`
   - uncomment `api/free_theory/compiler.rs`
   - `core::automata::*` - uncomment `binding_lhs_automaton`, `copy_rhs_automaton`, and `trivial_rhs_automata`.

- rewriter
  - rewriting methods on `core::rewriting_context::context::RewritingContext`
  - `StateTransitionGraph`
- Incorporate the remaining commented automata in the free theory: 
  * FreeFast3RHSAutomaton
  * FreeFast2RHSAutomaton
  * FreeTernaryRHSAutomaton
  * FreeBinaryRHSAutomaton
  * FreeUnaryRHSAutomaton
  * FreeNullaryRHSAutomaton

# Stubs

- Implement `StateTransitionGraph`


# Saved for later

- `AUExtensionInfo`, `AUCExtensionInfo`, `SExtensionInfo`
- `core::rewriting_context::trace`
- `core::rewriting_context::debugger`

# Issues

## Overwriting in place
Overwriting in-place is really problematic with fat pointers, because you aren't 
overwriting the vtable pointer in the fat pointer. If you can guarantee that
the vtable/theory doesn't change–which you can't–then it's ok.

One option is to store the vtable pointer in `DagNodeCore` (or just use
the theory tag) and reconstruct the fat pointer in `UnsafePointer`.

## `SymbolType`

The same symbol can have different `SymbolType`s in different modules. So the symbol 
type of a symbol is maintained in the module, not in the symbol itself. 
