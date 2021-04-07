# py_literal

[![Continuous integration](https://github.com/jturner314/py_literal/actions/workflows/ci.yml/badge.svg)](https://github.com/jturner314/py_literal/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/jturner314/py_literal/branch/master/graph/badge.svg)](https://codecov.io/gh/jturner314/py_literal)
[![Dependencies status](https://deps.rs/repo/github/jturner314/py_literal/status.svg)](https://deps.rs/repo/github/jturner314/py_literal)
[![Crate](https://img.shields.io/crates/v/py_literal.svg)](https://crates.io/crates/py_literal)
[![Documentation](https://docs.rs/py_literal/badge.svg)](https://docs.rs/py_literal)

This is a pure-Rust crate for parsing/formatting [Python literals]. See the
[documentation](https://docs.rs/py_literal) for more information.

[Python literals]: https://docs.python.org/3/reference/lexical_analysis.html#literals

**This crate is a work-in-progress.** The goal is for the parser to support
everything [`ast.literal_eval()`] does, but it supports only a subset. See the
docs for the `FromStr` implementation for `Value` for details.

[`ast.literal_eval()`]: https://docs.python.org/3/library/ast.html#ast.literal_eval

## Releases

* **0.4.0**

  * Updated `num-bigint` and `num-complex` dependencies to `0.4`.

* **0.3.0**

  * Updated `num-bigint` and `num-complex` dependencies to `0.3`.
  * Disabled default features of `num-complex` and `num-traits` dependencies.
  * Disabled `std` feature of `num-bigint` dependency.
  * Bumped required Rust version to 1.42.

* **0.2.2**

  * Updated `pest` and `pest_derive` dependencies to `2.0`, by @nagisa.

* **0.2.1**

  * Added `.is_*()` and `.as_*()` methods to `Value`.
  * Updated to the new style of `Error`. (Implemented `source`, and removed the
    non-default implementations of `description` and `cause`.)
  * Bumped required Rust version to 1.33.

* **0.2.0**

  * Updated `num-*` dependencies to 0.2.
  * Switched from depending on all of `num` to depending on the individual
    `num-*` crates.

* **0.1.1**

  * Improved crate metadata and documentation (no functional changes).

* **0.1.0**

  * Initial release.

## Contributing

Please feel free to create issues and submit PRs. PRs adding more tests would
be especially appreciated.

## License

Copyright 2018–2021 Jim Turner and `py_literal` developers

Licensed under the [Apache License, Version 2.0](LICENSE-APACHE), or the [MIT
license](LICENSE-MIT), at your option. You may not use this project except in
compliance with those terms.
