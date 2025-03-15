Built-in data types are in the Non-Algebraic Theory (NATheory). Their value is stored in both the `Term` and `DagNode`.

# Multithreading

The "multithreading" feature does nothing. It's not clear how multithreading could be implemented. Could multiple terms be rewritten simultaneously? 

- Many internal data structures would require guarded access.
- GC is not thread safe. All threads would need to simultaneously be at a safe point for GC to occur. 

# Hash Consing

Only `DagNode` tree structures use structural sharing.

# Garbage Collection

`DagNode`s and the data they own are garbage collected. Root nodes need to be tracked. Things that own `DagNode`s need to store them in a root container.
