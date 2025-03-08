# mod2 - A "Mini Maude" Term Rewriting System

Implements a small OBJ-like language for order-sorted term rewriting. This is the
front end. The algorithms for matching and rewriting are implemented in mod2-lib.

## The language

The language resembles and borrows concepts from
[Maude](https://maude.cs.illinois.edu/wiki/The_Maude_System) but is much smaller and
simpler. Items are organized into (possibly nested) namespaces called modules. There is
an implicit global module (unlike Maude). Expressions are formed from _symbols_, which
can be "nullary" constants, or _functors_ of specified or variadic arity, or _variables_.

```maude
symbol f [assoc, comm, id(h)];
symbol h/3 :: Int;
variable Z;
```

Expressions can have _sorts_, what other languages call types. Sorts are partially ordered
according to sort relations. We call a connected component in the lattice of sorts a _kind_.

```maude
sort A < B;
sort A < C;
sort B < D;
sort C < D;
```

The language includes _rules_ (left-to-right), _equations_ (bidirectional),
and _membership_ axioms, which put constraints on the sorts of expressions.

```maude
rule minus(s(X), s(Y)) => minus(X, Y);
equation plus(X, Y) = plus(Y, X);
membership f(X, h(Y, g(Z, Z), f(X))) :: NzNat -> NzNat -> NzNat if X := Y;
```

All of these may have side conditions, which constrains when the statement can apply. 

# License and Authorship

Copyright © 2025 Robert Jacobson. This software is distributed under the terms of the
[MIT license](LICENSE-MIT) or the [Apache 2.0 license](LICENSE-APACHE) at your preference.
