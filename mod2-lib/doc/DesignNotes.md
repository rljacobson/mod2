# Design

Much of mod2-lib follows Maude's implementation very closely. There are some important differences:

- Strategies are not implemented.
- Object oriented features are not implemented.
- There isn't a semantics for modules and imports yet, so they are not implemented.
- Since mod2-lib is meant to only implement the internal algorithms, details specific to the Maude language itself are
  either not implemented or are different.
- In particular, features related to Maude's sophisticated customizable syntax are not implemented.
- In Maude, `HashConsSet`s return an ID number for a DAG node. We just return the pointer. This might change if we need
  to swap out the node dynamically without invalidating already hash consed nodes.

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

# Notes

Built-in data types are in the Non-Algebraic Theory (NATheory). Their value is stored in both the `Term` and `DagNode`.

# Ownership and memory management

Things that are owned are generally owned in a `Box<Thing>`, whereas things that point to them do so with an `Unsafe<Thing>`. 

## `Terms`
Terms begin as a representation of the syntactic expression. They are then "normalized," which might transform an expression. The things that own terms own them in a `Box<Term>`. That means during normalization the term owner needs to replace the original term with its normalized version. Since things that don't own terms point to them with an `Unsafe<Term>`, all `Unsafe<Term>`'s are invalidated after normalization. This is obviously problematic, so we generally don't create `Unsafe<Term>` pointers prior to normalization.

# Hash Consing

Only `DagNode` tree structures use structural sharing.

# Garbage Collection

`DagNode`s and the data they own are garbage collected. Root nodes need to be tracked. Things that own `DagNode`s need to store them in a root container.

# Multithreading

The "multithreading" feature does nothing. It's not clear how multithreading could be implemented. Could multiple terms be rewritten simultaneously?

- Many internal data structures would require guarded access.
- GC is not thread safe. All threads would need to simultaneously be at a safe point for GC to occur. 
- However, GC allocation was written with multithreading in mind. It *should* be reentrant. 
