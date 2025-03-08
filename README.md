# `mod2-lib`: A pattern matching and term rewriting library

- The `mod2-lib` library builds on lessons learned in previous experiments to bring advanced pattern matching algorithms 
  to Rust.

- The `mod2` crate is a small Maude-like language meant to exercise the algorithms in `mod2-lib` and stand as a thorough
  example of how to use `mod2-lib`.

- The `mod2-abs` crate is an abstraction layer over backend implementations of various generally useful utilities used 
  throughout.

This project is a work in progress. For a more complete work, check out [Loris](https://github.com/rljacobson/loris).

# Status

- [X] Lexer & parser
- [X] M-expression
- [X] symbol declarations
- [X] modules syntax
- [ ] Module & submodule semantics
- [ ] custom operators
- [ ] Imperative Language
  - [ ]  match
  - [ ]  match_all
  - [ ]  unify
  - [ ]  replace
  - [ ]  replace_all

# License and Authorship

Copyright © 2025 Robert Jacobson. This software is distributed under the terms of the
[MIT license](LICENSE-MIT) or the [Apache 2.0 license](LICENSE-APACHE) at your preference.
