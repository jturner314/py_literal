use crate::Value;
use num_complex as numc;
use std::error::Error;
use std::fmt;
use std::io;

/// Error formatting a Python literal.
#[derive(Debug)]
pub enum FormatError {
    /// An error caused by the writer.
    Io(io::Error),
    /// The literal contained an empty set.
    ///
    /// There is no literal representation of an empty set in Python. (`{}`
    /// represents an empty `dict`.)
    EmptySet,
}

impl Error for FormatError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use FormatError::*;
        match self {
            Io(err) => Some(err),
            EmptySet => None,
        }
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FormatError::*;
        match self {
            Io(err) => write!(f, "I/O error: {}", err),
            EmptySet => write!(f, "unable to format empty set literal"),
        }
    }
}

impl From<io::Error> for FormatError {
    fn from(err: io::Error) -> FormatError {
        FormatError::Io(err)
    }
}

impl Value {
    /// Formats the value as an ASCII string.
    pub fn format_ascii(&self) -> Result<String, FormatError> {
        let mut out = Vec::new();
        self.write_ascii(&mut out)?;
        assert!(out.is_ascii());
        Ok(unsafe { String::from_utf8_unchecked(out) })
    }

    /// Writes the value as ASCII.
    ///
    /// This implementation performs a lot of small writes. If individual
    /// writes are expensive (e.g. if the writer is a [`TcpStream`]), it would
    /// be a good idea to wrap the writer in a [`BufWriter`] before passing it
    /// to `.write_ascii()`.
    ///
    /// [`TcpStream`]: https://doc.rust-lang.org/std/net/struct.TcpStream.html
    /// [`BufWriter`]: https://doc.rust-lang.org/std/io/struct.BufWriter.html
    pub fn write_ascii<W: io::Write>(&self, w: &mut W) -> Result<(), FormatError> {
        match *self {
            Value::String(ref s) => {
                w.write_all(b"'")?;
                for c in s.chars() {
                    match c {
                        '\\' => w.write_all(br"\\")?,
                        '\r' => w.write_all(br"\r")?,
                        '\n' => w.write_all(br"\n")?,
                        '\'' => w.write_all(br"\'")?,
                        c if c.is_ascii() => w.write_all(&[c as u8])?,
                        c => match c as u32 {
                            n @ 0..=0xff => write!(w, r"\x{:0>2x}", n)?,
                            n @ 0..=0xffff => write!(w, r"\u{:0>4x}", n)?,
                            n @ 0..=0xffff_ffff => write!(w, r"\U{:0>8x}", n)?,
                        },
                    }
                }
                w.write_all(b"'")?;
            }
            Value::Bytes(ref bytes) => {
                w.write_all(b"b'")?;
                for byte in bytes {
                    match *byte {
                        b'\\' => w.write_all(br"\\")?,
                        b'\r' => w.write_all(br"\r")?,
                        b'\n' => w.write_all(br"\n")?,
                        b'\'' => w.write_all(br"\'")?,
                        b if b.is_ascii() => w.write_all(&[b])?,
                        b => write!(w, r"\x{:0>2x}", b)?,
                    }
                }
                w.write_all(b"'")?;
            }
            Value::Integer(ref int) => write!(w, "{}", int)?,
            Value::Float(float) => {
                // Use scientific notation to make this unambiguously a float.
                write!(w, "{:e}", float)?;
            }
            Value::Complex(numc::Complex { re, im }) => {
                write!(w, "{}{:+}j", re, im)?;
            }
            Value::Tuple(ref tup) => {
                w.write_all(b"(")?;
                match tup.len() {
                    0 => (),
                    1 => {
                        tup[0].write_ascii(w)?;
                        w.write_all(b",")?;
                    }
                    _ => {
                        tup[0].write_ascii(w)?;
                        for value in &tup[1..] {
                            w.write_all(b", ")?;
                            value.write_ascii(w)?;
                        }
                    }
                }
                w.write_all(b")")?;
            }
            Value::List(ref list) => {
                w.write_all(b"[")?;
                if !list.is_empty() {
                    list[0].write_ascii(w)?;
                    for value in &list[1..] {
                        w.write_all(b", ")?;
                        value.write_ascii(w)?;
                    }
                }
                w.write_all(b"]")?;
            }
            Value::Dict(ref dict) => {
                w.write_all(b"{")?;
                if !dict.is_empty() {
                    dict[0].0.write_ascii(w)?;
                    w.write_all(b": ")?;
                    dict[0].1.write_ascii(w)?;
                    for elem in &dict[1..] {
                        w.write_all(b", ")?;
                        elem.0.write_ascii(w)?;
                        w.write_all(b": ")?;
                        elem.1.write_ascii(w)?;
                    }
                }
                w.write_all(b"}")?;
            }
            Value::Set(ref set) => {
                if set.is_empty() {
                    return Err(FormatError::EmptySet);
                } else {
                    w.write_all(b"{")?;
                    set[0].write_ascii(w)?;
                    for value in &set[1..] {
                        w.write_all(b", ")?;
                        value.write_ascii(w)?;
                    }
                    w.write_all(b"}")?;
                }
            }
            Value::Boolean(b) => {
                if b {
                    w.write_all(b"True")?;
                } else {
                    w.write_all(b"False")?;
                }
            }
            Value::None => w.write_all(b"None")?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
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
        assert_eq!("1+3j", format!("{}", Complex(numc::Complex::new(1., 3.))));
        assert_eq!("1-3j", format!("{}", Complex(numc::Complex::new(1., -3.))));
        assert_eq!("-1+3j", format!("{}", Complex(numc::Complex::new(-1., 3.))));
        assert_eq!(
            "-1-3j",
            format!("{}", Complex(numc::Complex::new(-1., -3.)))
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
            "{'foo': [1, True], {2+3j}: 4}",
            format!(
                "{}",
                Dict(vec![
                    (
                        String("foo".into()),
                        List(vec![Integer(1.into()), Boolean(true)]),
                    ),
                    (
                        Set(vec![Complex(numc::Complex::new(2., 3.))]),
                        Integer(4.into()),
                    ),
                ])
            ),
        );
    }
}
