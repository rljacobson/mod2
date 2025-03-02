# Design of `DagNode`

The `DagNode` is the heart of the engine. Speed hinges on efficient management of `DagNode` objects. Their creation,
reuse, and destruction are managed by an arena based garbage collecting allocator which relies on the fact that
every `DagNode` is of the same size. Since `DagNode`s can be of different types and have arguments, we make careful use
of transmute and bitflags.

The following compares Maude's `DagNode` to our implementation here.

|                | Maude                                        | mod2lib                  |
|:---------------|:---------------------------------------------|:-------------------------|
| size           | Fixed 3 word size                            | Fixed size enum variant  |
| tag            | implicit via vtable pointer                  | enum discriminant        |
| flags          | `MemoryInfo` in first word                   | `BitFlags` field         |
| shared impl    | base class impl                              | enum impl                |
| specialization | virtual function calls                       | match on variant in impl |
| args           | `reinterpret_cast` of 2nd word based on flag | Nested enum              |

