# Design Notes

## The type system

Sorts are named types that do not necessarily describe a data layout, that is,
they can be purely abstract. Right now, sorts cannot describe functor types.
Internally, a type is a `SortSpec` (a sort specification), which is either a
sort or a functor (or either of the special marker types `None` or `Any`).

Issues to consider:

 1. Sorts cannot be functions.
 2. Sort specs cannot be named in general, though they can be just a single named sort.
 3. Both sorts and sort specs lack native support for variadic list-like types. (We don't have a syntax for variadic
    functors yet.) List-like types can still be defined, but it's not ergonomic.

There is also what Maude calls a symbol's `SymbolType`, which stores the symbol's attributes (precedence, gather,
associative, etc.) and built-in type (if not "`Standard`"). The `SymbolType` seems to me to be type information
internal to Maude's implementation rather than intended to be part of the exposed type system.

It would be nice to have a more orthogonal type system, but it's not yet
clear what the sort solving algorithms require, and my priority right now is
to first understand the algorithms over and above writing the perfect system.

## The module system

### Unanswered questions

#### Are `Kind`s used after construction?

If `Kind`s have a use after the construction of the adjacency lists in the `Sorts` and the
assignment of each `Sort`'s `Sort.kind` field, then we need to keep them around. But so far I
don't see a use for them. I don't have a good understanding of how they are used during runtime.

#### Are items in a parent module automatically available within a submodule?

If so:

 - outer namespaces need to be searched to resolve names before inner namespaces
 - implicitly created resolvents like sorts are created from outer namespaces inward

If not:

 - how are names imported into the current scope? E.g. are all items of a module imported, or can one import just a
   subset?
 - If only a subset of items is imported, are pre-equations automatically imported?
 - What happens when names collide?


## Object Ownership

Module items are owned by the `Module` in which they are defined. References that can potentially create cycles,
such as the references within `Sort`s to other `Sort`s, are `Weak` references. Other references, like references
within `Term`s to `Symbols`, can be normal `Rc`s or `RcCell`s.

| Struct     | Owner     | Weak Referents     | Strong Referents                  |
|:-----------|:----------|:-------------------|:----------------------------------|
| `Sort`     | `Module`  | `Sort`             | `SortSpec`, `ConnectedComponent`  |
| `Symbol`   | `Module`  |                    | `Term`                            |
| `Module`   | `Module`  | `Sort`, `Symbol`,  |                                   |


```plantuml
class Sort


class Symbol {
    theory_symbol: TheorySymbol
    sort: Sort
}
class Term {
    theory_term: TheoryTerm
    symbol: Symbol
    args: Vec<Term>
}
abstract TheorySymbol
abstract TheoryTerm


Term --* Symbol
TheorySymbol *-- Symbol
Sort *-- Symbol

TheorySymbol <|-- FreeSymbol
TheorySymbol <|-- VariableSymbol
TheorySymbol <|-- ACUSymbol

TheoryTerm *-- Term

TheoryTerm <|-- FreeTerm
TheoryTerm <|-- VariableTerm
TheoryTerm <|-- ACUTerm

```

## Confused Maude Terminology

### Syntactic Structures

Some concepts in Maude are referred to in multiple ways, which can be confusing. This is my attempt to disentangle everything.[^strict]

[^strict]: I might introduce my own terms or syntax, so this is not strictly as found in Maude.

Maude calls the entire conjunction of conjunctands the condition, while individual conjunctands it calls a
`ConditionFragment`. I just use the word `Condition` for a conjunctand and `Conditions` for the conjunction of all
conjunctands.

| Concept            | Synonym                              | Relevant Syntax               |
|:-------------------|:-------------------------------------|:------------------------------|
| Sort Declaration   | Subsorts. Supersorts                 | `sort A < B;`                 |
| Membership Axiom   | Sort Constraint                      | `membership  X : Y if X > 0;` |
| Equality Condition | -none-                               | `…if X = Y…`                  |
| Sort Constraint    | Membership Axiom<br/>Sort Membership | `…if X : Y…`                  |
| Match Condition    | Assignment Condition                 | `…if X := Y…`                 |
| Rewrite Condition  | Rule Condition                       | `…if X => Y…`                 |

A *kind* in Maude refers specifically to the error supersort of a connected component. I use the word to refer to the connected component itself.

### Concepts

| Name               | Meaning                                                                                       |
|:-------------------|:-----------------------------------------------------|
| ConnectedComponent | Kind, which is a connected component of the lattice induced by the subsort relation on sorts. |
| Symbol type | This seems to be the part of the symbol's type that isn't encoded in the symbol's sort.       |


### Equational Theories

_Unitary_ means _with identity_. An identity (a unit) can be a left identity or a right identity (or both).

| Name           | Meaning                                              |
|:---------------|:-----------------------------------------------------|
| Unit (Unitary) | Identity, `e*x - x*e = x`                            |
| Idempotent     | `f(f(x)) = f(x)` (e.g. `AND` in `BOOL`)              |
| AU             | Associative with Unit (identity)                     |
| ACU            | Associative Commutative with Unit (identity)         |
| CUI            | Commutative with Unit, Idempotent                    |
| Free           | Theory free from additional relations or constraints |
