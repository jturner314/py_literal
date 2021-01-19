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
//! # fn main() -> Result<(), py_literal::ParseError> {
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
//! ```

mod format;
#[macro_use]
mod parse_macros;
mod parse;

pub use crate::format::FormatError;
pub use crate::parse::ParseError;

use num_bigint as numb;
use num_complex as numc;
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
    /// When formatting, backslash escapes are used to ensure the result
    /// contains only ASCII chars.
    String(String),
    /// Python byte sequence (`bytes`). When parsing, backslash escapes are
    /// interpreted. When formatting, backslash escapes are used to ensure the
    /// result contains only ASCII chars.
    Bytes(Vec<u8>),
    /// Python integer (`int`). Python integers have unlimited precision, so we
    /// use `BigInt`.
    Integer(numb::BigInt),
    /// Python floating-point number (`float`). The representation and
    /// precision of the Python `float` type varies by the machine where the
    /// program is executing, but `f64` should be good enough.
    Float(f64),
    /// Python complex number (`complex`). The Python `complex` type contains
    /// two `float` values.
    Complex(numc::Complex<f64>),
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        // TODO: is there a better way to do this?
        write!(f, "{}", self.format_ascii().map_err(|_| fmt::Error)?)
    }
}

impl Value {
    /// Returns `true` if `self` is `Value::String`. Returns `false` otherwise.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// If `self` is `Value::String`, returns the associated string. Returns `None` otherwise.
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Value::String(string) => Some(string),
            _ => None,
        }
    }

    /// Returns `true` if `self` is `Value::Bytes`. Returns `false` otherwise.
    pub fn is_bytes(&self) -> bool {
        matches!(self, Value::Bytes(_))
    }

    /// If `self` is `Value::Bytes`, returns the associated bytes. Returns `None` otherwise.
    pub fn as_bytes(&self) -> Option<&Vec<u8>> {
        match self {
            Value::Bytes(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Returns `true` if `self` is `Value::Integer`. Returns `false` otherwise.
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    /// If `self` is `Value::Integer`, returns the associated integer. Returns `None` otherwise.
    pub fn as_integer(&self) -> Option<&numb::BigInt> {
        match self {
            Value::Integer(integer) => Some(integer),
            _ => None,
        }
    }

    /// Returns `true` if `self` is `Value::Float`. Returns `false` otherwise.
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    /// If `self` is `Value::Float`, returns the associated float. Returns `None` otherwise.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(float) => Some(*float),
            _ => None,
        }
    }

    /// Returns `true` if `self` is `Value::Complex`. Returns `false` otherwise.
    pub fn is_complex(&self) -> bool {
        matches!(self, Value::Complex(_))
    }

    /// If `self` is `Value::Complex`, returns the associated complex number. Returns `None` otherwise.
    pub fn as_complex(&self) -> Option<numc::Complex<f64>> {
        match self {
            Value::Complex(complex) => Some(*complex),
            _ => None,
        }
    }

    /// Returns `true` if `self` is `Value::Tuple`. Returns `false` otherwise.
    pub fn is_tuple(&self) -> bool {
        matches!(self, Value::Tuple(_))
    }

    /// If `self` is `Value::Tuple`, returns the associated data. Returns `None` otherwise.
    pub fn as_tuple(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Tuple(tuple) => Some(tuple),
            _ => None,
        }
    }

    /// Returns `true` if `self` is `Value::List`. Returns `false` otherwise.
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    /// If `self` is `Value::List`, returns the associated data. Returns `None` otherwise.
    pub fn as_list(&self) -> Option<&Vec<Value>> {
        match self {
            Value::List(list) => Some(list),
            _ => None,
        }
    }

    /// Returns `true` if `self` is `Value::Dict`. Returns `false` otherwise.
    pub fn is_dict(&self) -> bool {
        matches!(self, Value::Dict(_))
    }

    /// If `self` is `Value::Dict`, returns the associated data. Returns `None` otherwise.
    pub fn as_dict(&self) -> Option<&Vec<(Value, Value)>> {
        match self {
            Value::Dict(dict) => Some(dict),
            _ => None,
        }
    }

    /// Returns `true` if `self` is `Value::Set`. Returns `false` otherwise.
    pub fn is_set(&self) -> bool {
        matches!(self, Value::Set(_))
    }

    /// If `self` is `Value::Set`, returns the associated data. Returns `None` otherwise.
    pub fn as_set(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Set(set) => Some(set),
            _ => None,
        }
    }

    /// Returns `true` if `self` is `Value::Boolean`. Returns `false` otherwise.
    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// If `self` is `Value::Boolean`, returns the associated data. Returns `None` otherwise.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(boolean) => Some(*boolean),
            _ => None,
        }
    }

    /// Returns `true` if `self` is `Value::None`. Returns `false` otherwise.
    pub fn is_none(&self) -> bool {
        matches!(self, Value::None)
    }
}
