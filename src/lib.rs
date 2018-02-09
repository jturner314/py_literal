//! This crate provides a type `Value` that represents a Python literal.
//! `Value` can be parsed from a string and formatted as a string.

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

use num::{BigInt, Complex};
use std::fmt;

/// Represents a Python literal expression.
///
/// This should be able to express everything that Python's
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
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: is there a better way to do this?
        write!(f, "{}", self.format_ascii().map_err(|_| fmt::Error)?)
    }
}
