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
