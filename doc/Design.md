# The module system

### Are items in a parent module automatically available within a submodule? 

If so: 

 - outer namespaces need to be searched to resolve names before inner namespaces
 - implicitly created resolvents like sorts are created from outer namespaces inward

If not: 

 - how are names imported into the current scope? E.g. are all items of a module imported, or can one import just a 
   subset?
 - If only a subset of items is imported, are pre-equations automatically imported?
 - What happens when names collide?


# Object Ownership

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

# Confused Maude Terminology

## Syntactic Structures

Some concepts in Maude are referred to in multiple ways, which can be confusing. This is my attempt to disentangle everything.[^1]

[^1]: I might introduce my own terms or syntax, so this is not strictly as found in Maude.

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



## Equational Theories

_Unitary_ means _with identity_. An identity (a unit) can be a left identity or a right identity (or both). 

| Name           | Meaning                                              |
|:---------------|:-----------------------------------------------------|
| Unit (Unitary) | Identity, `e*x - x*e = x`                            |
| Idempotent     | `f(f(x)) = f(x)` (e.g. `AND` in `BOOL`)              |
| AU             | Associative with Unit (identity)                     |
| ACU            | Associative Commutative with Unit (identity)         |
| CUI            | Commutative with Unit, Idempotent                    |
| Free           | Theory free from additional relations or constraints |
