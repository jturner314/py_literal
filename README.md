# py_literal

[![Build status](https://travis-ci.org/jturner314/py_literal.svg?branch=master)](https://travis-ci.org/jturner314/py_literal)
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

* **0.2.0** (not yet released)

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

Copyright 2018 Jim Turner

Licensed under the [Apache License, Version 2.0](LICENSE-APACHE), or the [MIT
license](LICENSE-MIT), at your option. You may not use this project except in
compliance with those terms.
