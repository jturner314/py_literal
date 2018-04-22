//! This crate provides a type [`Value`] that represents a [Python literal].
//! [`Value`] can be parsed from a string and formatted as a string.
//!
//! [`Value`]: enum.Value.html
//! [Python literal]: https://docs.python.org/3/reference/lexical_analysis.html#literals
//!
//! # Example
//!
//! ```
//! extern crate num;
//! extern crate py_literal;
//!
//! use num::{BigInt, Complex};
//! use py_literal::Value;
//!
//! # fn example() -> Result<(), py_literal::ParseError> {
//! // Parse a literal value from a string.
//! let value: Value = "{ 'foo': [5, (7e3,)], 2 - 5j: {b'bar'} }".parse()?;
//! assert_eq!(
//!     value,
//!     Value::Dict(vec![
//!         (
//!             Value::String("foo".to_string()),
//!             Value::List(vec![
//!                 Value::Integer(BigInt::from(5)),
//!                 Value::Tuple(vec![Value::Float(7e3)]),
//!             ]),
//!         ),
//!         (
//!             Value::Complex(Complex::new(2., -5.)),
//!             Value::Set(vec![Value::Bytes(b"bar".to_vec())]),
//!         ),
//!     ]),
//! );
//!
//! // Format a literal value as a string.
//! let formatted = format!("{}", value);
//! assert_eq!(
//!     formatted,
//!     "{'foo': [5, (7e3,)], 2-5j: {b'bar'}}",
//! );
//! # Ok(())
//! # }
//! # fn main() {
//! #     example().unwrap();
//! # }
//! ```

extern crate num;
extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate quick_error;

mod format;
#[macro_use]
mod parse_macros;
mod parse;

pub use format::FormatError;
pub use parse::ParseError;

use num::{BigInt, Complex};
use std::fmt;

/// Python literal.
///
/// This type should be able to express everything that Python's
/// [`ast.literal_eval()`] can evaluate, except for operators. Similar to
/// `literal_eval()`, addition and subtraction of numbers is supported in the
/// parser. However, binary addition and subtraction operators cannot be
/// formatted using `Value`.
///
/// [`ast.literal_eval()`]: https://docs.python.org/3/library/ast.html#ast.literal_eval
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Python string (`str`). When parsing, backslash escapes are interpreted.
    /// When formatting, backslash escapes are used to ensure the result is
    /// contains only ASCII chars.
    String(String),
    /// Python byte sequence (`bytes`). When parsing, backslash escapes are
    /// interpreted. When formatting, backslash escapes are used to ensure the
    /// result is contains only ASCII chars.
    Bytes(Vec<u8>),
    /// Python integer (`int`). Python integers have unlimited precision, so we
    /// use `BigInt`.
    Integer(BigInt),
    /// Python floating-point number (`float`). The representation and
    /// precision of the Python `float` type varies by the machine where the
    /// program is executing, but `f64` should be good enough.
    Float(f64),
    /// Python complex number (`complex`). The Python `complex` type contains
    /// two `float` values.
    Complex(Complex<f64>),
    /// Python tuple (`tuple`).
    Tuple(Vec<Value>),
    /// Python list (`list`).
    List(Vec<Value>),
    /// Python dictionary (`dict`).
    Dict(Vec<(Value, Value)>),
    /// Python set (`set`).
    Set(Vec<Value>),
    /// Python boolean (`bool`).
    Boolean(bool),
    /// Python `None`.
    None,
}

impl fmt::Display for Value {
    /// Formats the value as a Python literal.
    ///
    /// Currently, this just calls `self.format_ascii()`, but that may change
    /// in the future.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: is there a better way to do this?
        write!(f, "{}", self.format_ascii().map_err(|_| fmt::Error)?)
    }
}
