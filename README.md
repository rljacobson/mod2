# mod2 expression pattern matching library

This code is a new independent implementation of the pattern matching algorithms in
Maude and is being written in parallel to Mod. The purpose of this library is to resolve
architecture issues that I face writing Mod. Roughly speaking, this library is the
"front end," while the Mod code is the "back end," although in this case "front end"
corresponds more closely to the front and middle of a traditional compiler pipeline.

My approach to writing Mod is to start with internal algorithms and data structures.
On the other hand, mod2 starts with a front end, including a simple programming
language, and an internal representation of these frontend interfaces. The work on mod2
is largely independent of the Maude source code, the idea being that a natural code
shape will emerge with mod2. The work on Mod, on the other hand, is very dependent on
Maude's source code. Therefore, code in this repository can be released under a more
permissive license, either the MIT license or the Apache 2.0 license at your preference.

# Status

- [ ]  Lexer & parser
  - [X]  M-expression
  - [X]  symbol declarations
    - [ ]  variadic syntax
  - [ ]  Pre-equations
    - [X]  equations
    - [X]  rules
    - [X]  sort constraints
    - [ ]  strategy
  - [X]  modules syntax
  - [ ]  custom operators
- [X]  Term
- [ ]  sorts
- [ ]  Term -> DAG
- [ ]  rules
- [ ]  equations
- [ ]  free theory
  - [ ]  match
  - [ ]  match_all
  - [ ]  unify
  - [ ]  replace
  - [ ]  replace_all
- [ ]  associative theory (with unit)
- [ ]  commutative theory (with unit)
- [ ]  associative commutative theory (with unit)
- [ ]  Module & submodule semantics

# License and Authorship

Copyright (c) 2024 Robert Jacobson. This software library is distributed under the terms of the MIT license (MIT.txt) or the Apache 2.0 license (APACHE.txt) at your preference.
