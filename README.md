# py_literal

[![Build Status](https://travis-ci.org/jturner314/py_literal.svg?branch=master)](https://travis-ci.org/jturner314/py_literal)

This is a pure-Rust crate for parsing/formatting [Python literals]. See
[`src/lib.rs`](src/lib.rs) for more information.

[Python literals]: https://docs.python.org/3/reference/lexical_analysis.html#literals

**This crate is a work-in-progress.** It supports only a subset of Python
literals. See the docs for the `FromStr` implementation for `Value` for
details.

## Contributing

Please feel free to create issues and submit PRs. PRs adding more tests would
be especially appreciated.

## License

Copyright 2018 Jim Turner

Licensed under the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT
license](LICENSE-MIT), at your option. You may not use this project except in
compliance with those terms.
