## Terms and Symbols

The expressions on which the rewriting system acts are called terms. A *term* is a syntactic entity that can be a constant, a variable, or an application of a function symbol (or *functor*) to a sequence of other terms. In the context of algebraic specification and term rewriting systems, terms are the primary objects of manipulation and computation. Symbols are fundamental elements that serve as building blocks for constructing terms.

A symbol can represent either a constant or a function (or *functor*) that takes other terms as arguments. These symbols are defined within a module's signature, which outlines the structure and allowable operations within that module.

### Types of Symbols

- **Constant Symbols**: These are the simplest form of symbols, representing atomic values or entities that do not require arguments. Examples include numbers or strings. A symbol can also be any named item that stands alone without further decomposition, which you can think of as a function of arity zero, that is, a function that takes no arguments.
- **Function Symbols**: These symbols represent operations or functions that take a specified number of arguments (their *arity*) and produce a term. Function symbols are used to construct more complex terms from simpler ones. The arity of a function symbol can be zero, in which case it is also considered a constant.

We will use the terms “function symbol”, “function”, “operator symbol”, “operator”, and “operation” interchangeably.

### Role in Terms

Symbols are used to create terms, which are the primary data manipulated by Maude's rewriting engine. A term is either a single symbol or an application of a function symbol to a sequence of argument terms, each of which is also a term. The composition of these terms forms the basis for Maude's computational model.

### Sorts and Symbols

Each symbol is associated with a *sort*, which is a type that classifies the symbol within the type system of the module. Sorts help to define the domain of values or terms over which the symbols can operate, providing a way to enforce type correctness and to organize the symbols into a meaningful hierarchy.

- **Sorts of Constant Symbols**: The sort of a constant symbol indicates the type of entity it represents.
- **Sorts of Function Symbols**: For function symbols, the sort includes not only the type of the term they produce but also the types of the arguments they accept. This forms part of the function symbol's signature, which specifies its arity along with the sorts of its input and output.

**Example:**

The following defines two symbols of sort `Nat`[^1].

```maude
symbol a: Nat;
symbol b: Nat;
```

[1]: The `Nat` sort is part of the standard library. It models natural numbers, that is, nonnegative integers.

### Function Signatures

A *function signature* provides a formal description of a function symbol, specifying the types of arguments it accepts and the type of term it produces. It serves as a blueprint for how the function can be applied within terms and dictates the nature of computations involving that function.

#### Components

A function signature consists of three main components:

1. **Function Symbol**: The name or identifier representing the function, which is used in terms to denote the specific operation or computation.
2. **Argument Sorts**: A list of sorts, each corresponding to an expected type of an argument that the function takes. The number of sorts in this list defines the function's arity, indicating how many arguments the function requires.
3. **Return Sort**: The sort that represents the type of the result produced by the function when applied to its arguments.

#### Syntax

A function signature is typically written in the form:

```
symbol <function-symbol> :: <arg-sort1> ... <arg-sortN> -> <return-sort> ;
```

For example, a function signature for `add` might look like this:

```
symbol add :: Nat Nat -> Nat;
```

This signature states that `add` is a binary function symbol (arity of 2) that operates on natural numbers (`Nat`) and produces a natural number as a result.

The function signature ensures that terms are constructed correctly and that the computational semantics are well-defined. If a function is used in a term without applying any arguments or is applied to the wrong number or sorts of arguments, an error will be reported. 
