# In Progress

# Next Steps

- compiler
   - `Term::compile_lhs`, `Term::compile_rhs`
 
- rewriter
  - rewriting methods on `core::rewriting_context::context::RewritingContext`

# Saved for later

- `core::rewriting_context::trace`
- `core::rewriting_context::debugger`

# Issues

## Overwriting in place
Overwriting in-place is really problematic with fat pointers, because you aren't 
overwriting the vtable pointer in the fat pointer. If you can guarantee that
the vtable/theory doesn't change–which you can't–then it's ok.

One option is to store the vtable pointer in `DagNodeCore` (or just use
the theory tag) and reconstruct the fat pointer in `UnsafePointer`.
