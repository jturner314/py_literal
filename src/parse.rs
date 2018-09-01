use Value;
use num_bigint as numb;
use num_complex as numc;
use num_traits::{Num, ToPrimitive};
use pest::iterators::Pair;
use pest::Parser as ParserTrait;
use std::error::Error;
use std::num::ParseFloatError;
use std::str::FromStr;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("grammar.pest");

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct Parser;

quick_error! {
    /// Error parsing a Python literal.
    #[derive(Debug)]
    pub enum ParseError {
        /// A syntax error.
        Syntax(msg: String) {
            description("syntax error")
            display(x) -> ("{}: {}", x.description(), msg)
        }
        /// An illegal escape sequence in a string or bytes literal.
        IllegalEscapeSequence(msg: String) {
            description("illegal escape sequence in string or bytes")
            display(x) -> ("{}: {}", x.description(), msg)
        }
        /// An error parsing a float. This might happen if the mantissa or
        /// exponent in the float literal has too many digits.
        ParseFloat(err: ParseFloatError) {
            description("float parsing error")
            display(x) -> ("{}: {}", x.description(), err)
            cause(err)
            from()
        }
        /// An error in a numeric cast. For example, this might occur while
        /// adding an integer and float if the integer is too large to fit in a
        /// float.
        NumericCast(old: String, new_type: String) {
            description("error casting number")
            display(x) -> ("{}: {} to {}", x.description(), old, new_type)
        }
    }
}

impl FromStr for Value {
    type Err = ParseError;

    /// Parses a `Value` from a Python literal. The goal is for the parser to
    /// support everything [`ast.literal_eval()`] does. A few things haven't
    /// been implemented yet:
    ///
    /// * `r`/`R` and `u`/`U` prefixes for string and bytes literals.
    /// * [string literal concatenation]
    /// * newlines (except in string literals)
    /// * parentheses (except as tuple delimiters)
    /// * Unicode name escapes in strings (`\N{name}`)
    ///
    /// Note that the parser is limited to Python *literals*, not the full
    /// Python AST, so many things are not supported, such as:
    ///
    /// * identifiers
    /// * formatted string literals (`f`/`F` prefix)
    /// * binary operators (except for `+` and `-` on numeric literals)
    /// * function calls
    ///
    /// [`ast.literal_eval()`]: https://docs.python.org/3/library/ast.html#ast.literal_eval
    /// [string literal concatenation]: https://docs.python.org/3/reference/lexical_analysis.html#string-literal-concatenation
    fn from_str(s: &str) -> Result<Self, ParseError> {
        let mut parsed =
            Parser::parse(Rule::start, s).map_err(|e| ParseError::Syntax(format!("{}", e)))?;
        let (start,) = parse_pairs_as!(parsed, (Rule::start,));
        let (value,) = parse_pairs_as!(start.into_inner(), (Rule::value,));
        parse_value(value)
    }
}

fn parse_string_escape_seq(escape_seq: Pair<Rule>) -> Result<char, ParseError> {
    debug_assert_eq!(escape_seq.as_rule(), Rule::string_escape_seq);
    let (seq,) = parse_pairs_as!(escape_seq.into_inner(), (_,));
    match seq.as_rule() {
        Rule::char_escape => Ok(match seq.as_str() {
            "\\" => '\\',
            "'" => '\'',
            "\"" => '"',
            "a" => '\x07',
            "b" => '\x08',
            "f" => '\x0C',
            "n" => '\n',
            "r" => '\r',
            "t" => '\t',
            "v" => '\x0B',
            _ => unreachable!(),
        }),
        Rule::octal_escape => ::std::char::from_u32(u32::from_str_radix(seq.as_str(), 8).unwrap())
            .ok_or_else(|| {
                ParseError::IllegalEscapeSequence(format!(
                    "Octal escape is invalid: \\{}",
                    seq.as_str()
                ))
            }),
        Rule::hex_escape | Rule::unicode_hex_escape => ::std::char::from_u32(
            u32::from_str_radix(&seq.as_str()[1..], 16).unwrap(),
        ).ok_or_else(|| {
            ParseError::IllegalEscapeSequence(format!("Hex escape is invalid: \\x{}", seq.as_str()))
        }),
        Rule::name_escape => Err(ParseError::IllegalEscapeSequence(
            "Unicode name escapes are not supported.".into(),
        )),
        _ => unreachable!(),
    }
}

fn parse_string(string: Pair<Rule>) -> Result<String, ParseError> {
    debug_assert_eq!(string.as_rule(), Rule::string);
    let (string_body,) = parse_pairs_as!(string.into_inner(), (_,));
    match string_body.as_rule() {
        Rule::short_string_body | Rule::long_string_body => {
            let mut out = String::new();
            for item in string_body.into_inner() {
                match item.as_rule() {
                    Rule::short_string_non_escape
                    | Rule::long_string_non_escape
                    | Rule::string_unknown_escape => out.push_str(item.as_str()),
                    Rule::line_continuation_seq => (),
                    Rule::string_escape_seq => out.push(parse_string_escape_seq(item)?),
                    _ => unreachable!(),
                }
            }
            Ok(out)
        }
        _ => unreachable!(),
    }
}

fn parse_bytes_escape_seq(escape_seq: Pair<Rule>) -> Result<u8, ParseError> {
    debug_assert_eq!(escape_seq.as_rule(), Rule::bytes_escape_seq);
    let (seq,) = parse_pairs_as!(escape_seq.into_inner(), (_,));
    match seq.as_rule() {
        Rule::char_escape => Ok(match seq.as_str() {
            "\\" => b'\\',
            "'" => b'\'',
            "\"" => b'"',
            "a" => b'\x07',
            "b" => b'\x08',
            "f" => b'\x0C',
            "n" => b'\n',
            "r" => b'\r',
            "t" => b'\t',
            "v" => b'\x0B',
            _ => unreachable!(),
        }),
        Rule::octal_escape => u8::from_str_radix(seq.as_str(), 8).map_err(|err| {
            ParseError::IllegalEscapeSequence(format!(
                "failed to parse \\{} as u8: {}",
                seq.as_str(),
                err,
            ))
        }),
        Rule::hex_escape => Ok(u8::from_str_radix(&seq.as_str()[1..], 16).unwrap()),
        _ => unreachable!(),
    }
}

fn parse_bytes(bytes: Pair<Rule>) -> Result<Vec<u8>, ParseError> {
    debug_assert_eq!(bytes.as_rule(), Rule::bytes);
    let (bytes_body,) = parse_pairs_as!(bytes.into_inner(), (_,));
    match bytes_body.as_rule() {
        Rule::short_bytes_body | Rule::long_bytes_body => {
            let mut out = Vec::new();
            for item in bytes_body.into_inner() {
                match item.as_rule() {
                    Rule::short_bytes_non_escape
                    | Rule::long_bytes_non_escape
                    | Rule::bytes_unknown_escape => out.extend_from_slice(item.as_str().as_bytes()),
                    Rule::line_continuation_seq => (),
                    Rule::bytes_escape_seq => out.push(parse_bytes_escape_seq(item)?),
                    _ => unreachable!(),
                }
            }
            Ok(out)
        }
        _ => unreachable!(),
    }
}

fn parse_number_expr(expr: Pair<Rule>) -> Result<Value, ParseError> {
    debug_assert_eq!(expr.as_rule(), Rule::number_expr);
    let mut result = Value::Integer(0.into());
    let mut neg = false;
    for pair in expr.into_inner() {
        match pair.as_rule() {
            Rule::minus_sign => neg = !neg,
            Rule::number => {
                let num = parse_number(pair)?;
                if neg {
                    result = sub_numbers(result, num).unwrap();
                } else {
                    result = add_numbers(result, num).unwrap();
                }
                neg = false;
            }
            _ => unreachable!(),
        }
    }
    Ok(result)
}

fn parse_number(number: Pair<Rule>) -> Result<Value, ParseError> {
    debug_assert_eq!(number.as_rule(), Rule::number);
    let (inner,) = parse_pairs_as!(number.into_inner(), (_,));
    match inner.as_rule() {
        Rule::imag => parse_imag(inner),
        Rule::float => Ok(Value::Float(parse_float(inner)?)),
        Rule::integer => Ok(Value::Integer(parse_integer(inner))),
        _ => unreachable!(),
    }
}

fn parse_integer(int: Pair<Rule>) -> numb::BigInt {
    debug_assert_eq!(int.as_rule(), Rule::integer);
    let (inner,) = parse_pairs_as!(int.into_inner(), (_,));
    match inner.as_rule() {
        Rule::bin_integer => {
            let digits: String = inner.into_inner().map(|digit| digit.as_str()).collect();
            numb::BigInt::from_str_radix(&digits, 2).expect(&format!(
                "failure parsing binary integer with digits {}",
                digits
            ))
        }
        Rule::oct_integer => {
            let digits: String = inner.into_inner().map(|digit| digit.as_str()).collect();
            numb::BigInt::from_str_radix(&digits, 8).expect(&format!(
                "failure parsing octal integer with digits {}",
                digits
            ))
        }
        Rule::hex_integer => {
            let digits: String = inner.into_inner().map(|digit| digit.as_str()).collect();
            numb::BigInt::from_str_radix(&digits, 16).expect(&format!(
                "failure parsing hexadecimal integer with digits {}",
                digits
            ))
        }
        Rule::dec_integer => {
            let digits: String = inner.into_inner().map(|digit| digit.as_str()).collect();
            digits
                .parse()
                .expect(&format!("failure parsing integer with digits {}", digits))
        }
        _ => unreachable!(),
    }
}

fn parse_float(float: Pair<Rule>) -> Result<f64, ParseError> {
    debug_assert_eq!(float.as_rule(), Rule::float);
    let (inner,) = parse_pairs_as!(float.into_inner(), (_,));
    let mut parsable = String::new();
    for pair in inner.into_inner().flatten() {
        match pair.as_rule() {
            Rule::digit => parsable.push_str(pair.as_str()),
            Rule::fraction => parsable.push('.'),
            Rule::pos_exponent => parsable.push('e'),
            Rule::neg_exponent => parsable.push_str("e-"),
            _ => (),
        }
    }
    Ok(parsable.parse()?)
}

fn parse_imag(imag: Pair<Rule>) -> Result<Value, ParseError> {
    debug_assert_eq!(imag.as_rule(), Rule::imag);
    let (inner,) = parse_pairs_as!(imag.into_inner(), (_,));
    let imag: f64 = match inner.as_rule() {
        Rule::float => parse_float(inner)?,
        Rule::digit_part => {
            let digits: String = inner.into_inner().map(|digit| digit.as_str()).collect();
            digits.parse()?
        }
        _ => unreachable!(),
    };
    Ok(Value::Complex(numc::Complex::new(0., imag)))
}

/// Parses a tuple, list, or set.
fn parse_seq(seq: Pair<Rule>) -> Result<Vec<Value>, ParseError> {
    debug_assert!([Rule::tuple, Rule::list, Rule::set].contains(&seq.as_rule()));
    seq.into_inner().map(|elem| parse_value(elem)).collect()
}

fn parse_dict(dict: Pair<Rule>) -> Result<Vec<(Value, Value)>, ParseError> {
    debug_assert_eq!(dict.as_rule(), Rule::dict);
    let mut out = Vec::new();
    for elem in dict.into_inner() {
        let (key, value) = parse_pairs_as!(elem.into_inner(), (Rule::value, Rule::value));
        out.push((parse_value(key)?, parse_value(value)?));
    }
    Ok(out)
}

fn parse_boolean(b: Pair<Rule>) -> bool {
    debug_assert_eq!(b.as_rule(), Rule::boolean);
    match b.as_str() {
        "True" => true,
        "False" => false,
        _ => unreachable!(),
    }
}

/// NumPy uses [`ast.literal_eval()`] to parse the header dictionary.
/// `literal_eval()` supports only the following Python literals: strings,
/// bytes, numbers, tuples, lists, dicts, sets, booleans, and `None`.
///
/// [`ast.literal_eval()`]: https://docs.python.org/3/library/ast.html#ast.literal_eval
fn parse_value(value: Pair<Rule>) -> Result<Value, ParseError> {
    debug_assert_eq!(value.as_rule(), Rule::value);
    let (inner,) = parse_pairs_as!(value.into_inner(), (_,));
    match inner.as_rule() {
        Rule::string => Ok(Value::String(parse_string(inner)?)),
        Rule::bytes => Ok(Value::Bytes(parse_bytes(inner)?)),
        Rule::number_expr => parse_number_expr(inner),
        Rule::tuple => Ok(Value::Tuple(parse_seq(inner)?)),
        Rule::list => Ok(Value::List(parse_seq(inner)?)),
        Rule::dict => Ok(Value::Dict(parse_dict(inner)?)),
        Rule::set => Ok(Value::Set(parse_seq(inner)?)),
        Rule::boolean => Ok(Value::Boolean(parse_boolean(inner))),
        Rule::none => Ok(Value::None),
        _ => unreachable!(),
    }
}

fn int_to_f64(int: numb::BigInt) -> Result<f64, ParseError> {
    int.to_f64()
        .ok_or_else(|| ParseError::NumericCast(format!("{}", int), "f64".into()))
}

/// Adds two numbers.
///
/// **Panics** if either of the arguments is not a number.
fn add_numbers(lhs: Value, rhs: Value) -> Result<Value, ParseError> {
    use self::Value::*;
    match (lhs, rhs) {
        (Integer(int1), Integer(int2)) => Ok(Integer(int1 + int2)),
        (Float(float1), Float(float2)) => Ok(Float(float1 + float2)),
        (Complex(comp1), Complex(comp2)) => Ok(Complex(comp1 + comp2)),
        (Integer(int), Float(float)) | (Float(float), Integer(int)) => {
            Ok(Float(int_to_f64(int)? + float))
        }
        (Integer(int), Complex(comp)) | (Complex(comp), Integer(int)) => {
            Ok(Complex(int_to_f64(int)? + comp))
        }
        (Float(float), Complex(comp)) | (Complex(comp), Float(float)) => Ok(Complex(float + comp)),
        _ => unimplemented!(),
    }
}

/// Subtracts two numbers.
///
/// **Panics** if either of the arguments is not a number.
fn sub_numbers(lhs: Value, rhs: Value) -> Result<Value, ParseError> {
    use self::Value::*;
    match (lhs, rhs) {
        (Integer(int1), Integer(int2)) => Ok(Integer(int1 - int2)),
        (Integer(int), Float(float)) => Ok(Float(int_to_f64(int)? - float)),
        (Integer(int), Complex(comp)) => Ok(Complex(int_to_f64(int)? - comp)),
        (Float(float), Integer(int)) => Ok(Float(float - int_to_f64(int)?)),
        (Float(float1), Float(float2)) => Ok(Float(float1 - float2)),
        (Float(float), Complex(comp)) => Ok(Complex(float - comp)),
        (Complex(comp), Integer(int)) => Ok(Complex(comp - int_to_f64(int)?)),
        (Complex(comp), Float(float)) => Ok(Complex(comp - float)),
        (Complex(comp1), Complex(comp2)) => Ok(Complex(comp1 - comp2)),
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_string_example() {
        for &(input, correct) in &[
            ("''", ""),
            (
                r#"'he\qllo\th\03o\x1bw\
a\n\rre\a\'\"y\u1234o\U00031234u'"#,
                "he\\qllo\th\x03o\x1bwa\n\rre\x07'\"y\u{1234}o\u{31234}u",
            ),
        ] {
            let mut parsed = Parser::parse(Rule::string, input)
                .unwrap_or_else(|err| panic!("failed to parse: {}", err));
            let s = parse_string(parse_pairs_as!(parsed, (Rule::string,)).0).unwrap();
            assert_eq!(s, correct);
        }
    }

    #[test]
    fn parse_bytes_example() {
        for &(input, correct) in &[
            ("b''", &b""[..]),
            (
                r#"b'he\qllo\th\03o\x1bw\
a\n\rre\a\'\"y\u1234o\U00031234u'"#,
                &b"he\\qllo\th\x03o\x1bwa\n\rre\x07'\"y\\u1234o\\U00031234u"[..],
            ),
        ] {
            let mut parsed = Parser::parse(Rule::bytes, input)
                .unwrap_or_else(|err| panic!("failed to parse: {}", err));
            let bytes = parse_bytes(parse_pairs_as!(parsed, (Rule::bytes,)).0).unwrap();
            assert_eq!(bytes, correct);
        }
    }

    #[test]
    fn parse_number_expr_example() {
        let input = "+-23 + 4.5 -+- -5j - 3e2 + 1.2 - 9";
        let mut parsed = Parser::parse(Rule::number_expr, input)
            .unwrap_or_else(|err| panic!("failed to parse: {}", err));
        let expr = parse_number_expr(parse_pairs_as!(parsed, (Rule::number_expr,)).0).unwrap();
        assert_eq!(
            expr,
            Value::Complex(-23. + 4.5 - numc::Complex::new(0., 5.) - 3e2 + 1.2 - 9.)
        );
    }

    #[test]
    fn parse_integer_example() {
        let inputs = ["0b_1001_0010_1010", "0o44_52", "0x9_2a", "2_346"];
        for input in &inputs {
            let mut parsed = Parser::parse(Rule::integer, input)
                .unwrap_or_else(|err| panic!("failed to parse: {}", err));
            let int = parse_integer(parse_pairs_as!(parsed, (Rule::integer,)).0);
            assert_eq!(int, numb::BigInt::from(2346));
        }
    }

    #[test]
    fn parse_float_example() {
        let input = "3_51.4_6e-2_7";
        let mut parsed = Parser::parse(Rule::float, input)
            .unwrap_or_else(|err| panic!("failed to parse: {}", err));
        let float = parse_float(parse_pairs_as!(parsed, (Rule::float,)).0).unwrap();
        assert_eq!(float, 351.46e-27);
    }

    #[test]
    fn parse_tuple_example() {
        use self::Value::*;
        for &(input, ref correct) in &[
            ("()", Tuple(vec![])),
            ("(5, )", Tuple(vec![Integer(5.into())])),
            ("(1, 2)", Tuple(vec![Integer(1.into()), Integer(2.into())])),
            ("(1, 2,)", Tuple(vec![Integer(1.into()), Integer(2.into())])),
        ] {
            let mut parsed = Parser::parse(Rule::value, input)
                .unwrap_or_else(|err| panic!("failed to parse: {}", err));
            let tuple = parse_value(parse_pairs_as!(parsed, (Rule::value,)).0).unwrap();
            assert_eq!(tuple, *correct);
        }
    }

    #[test]
    fn parse_list_example() {
        use self::Value::*;
        for &(input, ref correct) in &[
            ("[]", List(vec![])),
            ("[3]", List(vec![Integer(3.into())])),
            ("[5,]", List(vec![Integer(5.into())])),
            ("[1, 2]", List(vec![Integer(1.into()), Integer(2.into())])),
            (
                "[5, 6., \"foo\", 2+7j,]",
                List(vec![
                    Integer(5.into()),
                    Float(6.),
                    String("foo".into()),
                    Complex(numc::Complex::new(2., 7.)),
                ]),
            ),
        ] {
            let mut parsed = Parser::parse(Rule::value, input)
                .unwrap_or_else(|err| panic!("failed to parse: {}", err));
            let list = parse_value(parse_pairs_as!(parsed, (Rule::value,)).0).unwrap();
            assert_eq!(list, *correct);
        }
    }

    #[test]
    fn parse_dict_example() {
        use self::Value::*;
        for &(input, ref correct) in &[
            ("{}", Dict(vec![])),
            (
                "{ 3: None}",
                Dict(vec![(Integer(3.into()), None)]),
            ),
            (
                "{5: 6., \"foo\" : True, b'bar' :False }",
                Dict(vec![
                    (Integer(5.into()), Float(6.)),
                    (String("foo".into()), Boolean(true)),
                    (Bytes("bar".into()), Boolean(false)),
                ]),
            ),
        ] {
            let mut parsed = Parser::parse(Rule::value, input)
                .unwrap_or_else(|err| panic!("failed to parse: {}", err));
            let dict = parse_value(parse_pairs_as!(parsed, (Rule::value,)).0).unwrap();
            assert_eq!(dict, *correct);
        }
    }

    #[test]
    fn parse_set_example() {
        use self::Value::*;
        for &(input, ref correct) in &[
            ("{3}", Set(vec![Integer(3.into())])),
            ("{5,}", Set(vec![Integer(5.into())])),
            ("{1, 2}", Set(vec![Integer(1.into()), Integer(2.into())])),
        ] {
            let mut parsed = Parser::parse(Rule::value, input)
                .unwrap_or_else(|err| panic!("failed to parse: {}", err));
            let set = parse_value(parse_pairs_as!(parsed, (Rule::value,)).0).unwrap();
            assert_eq!(set, *correct);
        }
    }

    #[test]
    fn parse_list_of_tuples_example() {
        use self::Value::*;
        for &(input, ref correct) in &[
            (
                "[('big', '>i4'), ('little', '<i4')]",
                List(vec![
                    Tuple(vec![String("big".into()), String(">i4".into())]),
                    Tuple(vec![String("little".into()), String("<i4".into())]),
                ]),
            ),
            (
                "[(1, 2, 3), (4,)]",
                List(vec![
                    Tuple(vec![
                        Integer(1.into()),
                        Integer(2.into()),
                        Integer(3.into()),
                    ]),
                    Tuple(vec![Integer(4.into())]),
                ]),
            ),
        ] {
            let mut parsed = Parser::parse(Rule::value, input)
                .unwrap_or_else(|err| panic!("failed to parse: {}", err));
            let list = parse_value(parse_pairs_as!(parsed, (Rule::value,)).0).unwrap();
            assert_eq!(list, *correct);
        }
    }
}
