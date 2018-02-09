use Value;
use num;
use std::error::Error;
use std::fmt::{self, Write};

quick_error! {
    /// Error formatting a Python literal.
    #[derive(Debug)]
    pub enum FormatError {
        /// The literal contained an empty set.
        ///
        /// There is no literal representation of an empty set in Python. (`{}`
        /// represents an empty `dict`.)
        EmptySet {
            description("unable to format empty set literal")
            display(x) -> ("{}", x.description())
        }
        /// An error caused by the writer.
        Writer(err: fmt::Error) {
            description("error in format writer")
            display(x) -> ("{}", x.description())
            cause(err)
            from()
        }
    }
}

impl Value {
    pub fn format_ascii(&self) -> Result<String, FormatError> {
        let mut out = String::new();
        self.write_ascii(&mut out)?;
        Ok(out)
    }

    pub fn write_ascii<W: Write>(&self, mut w: W) -> Result<(), FormatError> {
        match *self {
            Value::String(ref s) => {
                w.write_str("'")?;
                for c in s.chars() {
                    match c {
                        '\\' => w.write_str(r"\\")?,
                        '\r' => w.write_str(r"\r")?,
                        '\n' => w.write_str(r"\n")?,
                        '\'' => w.write_str(r"\'")?,
                        c if c.is_ascii() => w.write_char(c)?,
                        c => match c as u32 {
                            n @ 0...0xff => write!(w, r"\x{:0>2x}", n)?,
                            n @ 0...0xffff => write!(w, r"\u{:0>4x}", n)?,
                            n @ 0...0xffffffff => write!(w, r"\U{:0>8x}", n)?,
                            _ => unreachable!(),
                        },
                    }
                }
                w.write_str("'")?;
            }
            Value::Bytes(ref bytes) => {
                w.write_str("b'")?;
                for byte in bytes {
                    match *byte {
                        b'\\' => w.write_str(r"\\")?,
                        b'\r' => w.write_str(r"\r")?,
                        b'\n' => w.write_str(r"\n")?,
                        b'\'' => w.write_str(r"\'")?,
                        b if b.is_ascii() => w.write_char(b.into())?,
                        b => write!(w, r"\x{:0>2x}", b)?,
                    }
                }
                w.write_str("'")?;
            }
            Value::Integer(ref int) => write!(w, "{}", int)?,
            Value::Float(float) => {
                // Use scientific notation to make this unambiguously a float.
                write!(w, "{:e}", float)?;
            }
            Value::Complex(num::Complex { re, im }) => {
                // Use scientific notation to make the parts unambiguously floats.
                write!(w, "{:e}{:+e}j", re, im)?;
            }
            Value::Tuple(ref tup) => {
                w.write_str("(")?;
                match tup.len() {
                    0 => (),
                    1 => write!(w, "{},", tup[0])?,
                    _ => {
                        write!(w, "{}", tup[0])?;
                        for value in &tup[1..] {
                            write!(w, ", {}", value)?;
                        }
                    }
                }
                w.write_str(")")?;
            }
            Value::List(ref list) => {
                w.write_str("[")?;
                if !list.is_empty() {
                    write!(w, "{}", list[0])?;
                    for value in &list[1..] {
                        write!(w, ", {}", value)?;
                    }
                }
                w.write_str("]")?;
            }
            Value::Dict(ref dict) => {
                w.write_str("{")?;
                if !dict.is_empty() {
                    write!(w, "{}: {}", dict[0].0, dict[0].1)?;
                    for elem in &dict[1..] {
                        write!(w, ", {}: {}", elem.0, elem.1)?;
                    }
                }
                w.write_str("}")?;
            }
            Value::Set(ref set) => {
                if set.is_empty() {
                    return Err(FormatError::EmptySet);
                } else {
                    w.write_str("{")?;
                    write!(w, "{}", set[0])?;
                    for value in &set[1..] {
                        write!(w, ", {}", value)?;
                    }
                    w.write_str("}")?;
                }
            }
            Value::Boolean(b) => {
                if b {
                    w.write_str("True")?;
                } else {
                    w.write_str("False")?;
                }
            }
            Value::None => w.write_str("None")?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use num;
    use super::*;

    #[test]
    fn format_string() {
        let value = Value::String("hello\th\x03\u{ff}o\x1bware\x07'y\u{1234}o\u{31234}u".into());
        let formatted = format!("{}", value);
        assert_eq!(
            formatted,
            "'hello\th\x03\\xffo\x1bware\x07\\'y\\u1234o\\U00031234u'"
        )
    }

    #[test]
    fn format_bytes() {
        let value = Value::Bytes(b"hello\th\x03\xffo\x1bware\x07'you"[..].into());
        let formatted = format!("{}", value);
        assert_eq!(formatted, "b'hello\th\x03\\xffo\x1bware\x07\\'you'")
    }

    #[test]
    fn format_complex() {
        use self::Value::*;
        assert_eq!(
            "1e0+3e0j",
            format!("{}", Complex(num::Complex::new(1., 3.)))
        );
        assert_eq!(
            "1e0-3e0j",
            format!("{}", Complex(num::Complex::new(1., -3.)))
        );
        assert_eq!(
            "-1e0+3e0j",
            format!("{}", Complex(num::Complex::new(-1., 3.)))
        );
        assert_eq!(
            "-1e0-3e0j",
            format!("{}", Complex(num::Complex::new(-1., -3.)))
        );
    }

    #[test]
    fn format_tuple() {
        use self::Value::*;
        assert_eq!("()", format!("{}", Tuple(vec![])));
        assert_eq!("(1,)", format!("{}", Tuple(vec![Integer(1.into())])));
        assert_eq!(
            "(1, 2)",
            format!("{}", Tuple(vec![Integer(1.into()), Integer(2.into())]))
        );
        assert_eq!(
            "(1, 2, 'hi')",
            format!(
                "{}",
                Tuple(vec![
                    Integer(1.into()),
                    Integer(2.into()),
                    String("hi".into()),
                ])
            ),
        );
    }

    #[test]
    fn format_list() {
        use self::Value::*;
        assert_eq!("[]", format!("{}", List(vec![])));
        assert_eq!("[1]", format!("{}", List(vec![Integer(1.into())])));
        assert_eq!(
            "[1, 2]",
            format!("{}", List(vec![Integer(1.into()), Integer(2.into())]))
        );
        assert_eq!(
            "[1, 2, 'hi']",
            format!(
                "{}",
                List(vec![
                    Integer(1.into()),
                    Integer(2.into()),
                    String("hi".into()),
                ])
            ),
        );
    }

    #[test]
    fn format_dict() {
        use self::Value::*;
        assert_eq!("{}", format!("{}", Dict(vec![])));
        assert_eq!(
            "{1: 2}",
            format!("{}", Dict(vec![(Integer(1.into()), Integer(2.into()))]))
        );
        assert_eq!(
            "{1: 2, 'foo': 'bar'}",
            format!(
                "{}",
                Dict(vec![
                    (Integer(1.into()), Integer(2.into())),
                    (String("foo".into()), String("bar".into())),
                ])
            ),
        );
    }

    #[test]
    #[should_panic]
    fn format_empty_set() {
        use self::Value::*;
        format!("{}", Set(vec![]));
    }

    #[test]
    fn format_set() {
        use self::Value::*;
        assert_eq!("{1}", format!("{}", Set(vec![Integer(1.into())])));
        assert_eq!(
            "{1, 2}",
            format!("{}", Set(vec![Integer(1.into()), Integer(2.into())]))
        );
        assert_eq!(
            "{1, 2, 'hi'}",
            format!(
                "{}",
                Set(vec![
                    Integer(1.into()),
                    Integer(2.into()),
                    String("hi".into()),
                ])
            ),
        );
    }

    #[test]
    fn format_nested() {
        use self::Value::*;
        assert_eq!(
            "{'foo': [1, True], {2e0+3e0j}: 4}",
            format!(
                "{}",
                Dict(vec![
                    (
                        String("foo".into()),
                        List(vec![Integer(1.into()), Boolean(true)]),
                    ),
                    (
                        Set(vec![Complex(num::Complex::new(2., 3.))]),
                        Integer(4.into()),
                    ),
                ])
            ),
        );
    }
}
