# Hash Consing

Tree structures for which we use structural sharing:
- `Term`
- `DagNode`
- Maybe `SortSpec`?

# Garbage Collection

`DagNode`s and the data they own are garbage collected. Root nodes need to be tracked. Things that own `DagNode`s need to store them in a root container.

- 

