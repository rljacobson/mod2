# `mod2-lib`: A pattern matching and term rewriting library

The `mod2-lib` library builds on lessons learned in writing [`Mod`](https://github.com/rljacobson/Mod) and
[`mod2`](https://github.com/rljacobson/mod2) to bring advanced pattern matching algorithms to Rust.

It is a work in progress. For a more complete work, check out [Loris](https://github.com/rljacobson/loris).

## Background

> **[Maude](https://github.com/SRI-CSL/Maude)** is a high-performance reflective language and system supporting both equational and rewriting logic
> specification and programming for a wide range of applications.

Maude is interesting in part because it implements some of the most performant and sophisticated pattern matching
algorithms that are known. Some of the algorithms are described across the literature. (See the
[Bibliography](../doc/Bibliography.md).) The most important references are:

* S. Eker, _Fast matching in combinations of regular equational theories_, Electronic Notes in Theoretical Computer
  Science, 1996,
  vol. 4, p. 90-109, ISSN 1571-0661, https://doi.org/10.1016/S1571-0661(04)00035-0.
* S. Eker,
  _Associative-commutative matching via bipartite graph matching_,
  Computer Journal, 38 (5) (1995), pp. 381-399

The algorithms are complicated. Maude is implemented in C++. The code is excellent. However, because Maude was
designed to be modular, and because of the algorithms in \[Eker 1996] that allow combinations of theories, the
algorithms for matching are somewhat obscured. In other words, you can't just copy and paste the algorithm into your
own code.

Thus, I am attempting to reimplement the algorithms in Rust and hopefully clarify some of the implementation details
at the same time.


## Status

- [ ]  Pre-equations
    - [X]  equations
    - [X]  rules
    - [X]  membership / sort constraints
    - [ ]  strategy
- [X]  Term
- [ ]  sorts
- [ ]  Term -> DAG
- [ ]  rules
- [ ]  equations
- [ ]  free theory
- [ ]  algorithms
    - [ ]  match
    - [ ]  match_all
    - [ ]  unify
    - [ ]  replace
    - [ ]  replace_all
- [ ]  associative theory (with unit)
- [ ]  commutative theory (with unit)
- [ ]  associative commutative theory (with unit)


## Building

From the workspace root you can run

```shell
cargo build --package mod2-lib
```

But that doesn't exactly do anything. The `mod2` package provides an OBJ3-like syntax as a front-end to the pattern 
matching / term rewriting system.

```shell
cargo build --package mod2
```

See the example(s) in `mod2/examples/`.

# Authorship and License

Copyright © 2025 Robert Jacobson. This software is distributed under the terms of the
[MIT license](LICENSE-MIT) or the [Apache 2.0 license](LICENSE-APACHE) at your preference.
