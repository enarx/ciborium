// SPDX-License-Identifier: Apache-2.0

use ciborium::{
    de::{from_reader, Error},
    ser::into_writer,
    value::Value,
};
use rstest::rstest;
use std::io::ErrorKind;

fn eof() -> Error<std::io::Error> {
    Error::Io(ErrorKind::UnexpectedEof.into())
}

#[rstest(bytes, error,
    // Invalid value
    case("1e", Error::Syntax(0)),

    // Indeterminate integers are invalid
    case("1f", Error::Syntax(0)),

    // Indeterminate integer in an array
    case("83011f03", Error::Syntax(2)),

    // Integer in a string continuation
    case("7F616101FF", Error::Syntax(3)),

    // Bytes in a string continuation
    case("7F61614101FF", Error::Syntax(3)),

    // Invalid UTF-8
    case("62C328", Error::Syntax(0)),

    // Invalid UTF-8 in a string continuation
    case("7F62C328FF", Error::Syntax(1)),

    // End of input in a head
    case("18", eof()),
    case("19", eof()),
    case("1a", eof()),
    case("1b", eof()),
    case("1901", eof()),
    case("1a0102", eof()),
    case("1b01020304050607", eof()),
    case("38", eof()),
    case("58", eof()),
    case("78", eof()),
    case("98", eof()),
    case("9a01ff00", eof()),
    case("b8", eof()),
    case("d8", eof()),
    case("f8", eof()),
    case("f900", eof()),
    case("fa0000", eof()),
    case("fb000000", eof()),

    // Definite-length strings with short data:
    case("41", eof()),
    case("61", eof()),
    case("5affffffff00", eof()),
    case("5bffffffffffffffff010203", eof()),
    case("7affffffff00", eof()),
    case("7b7fffffffffffffff010203", eof()),

    // Definite-length maps and arrays not closed with enough items:
    case("81", eof()),
    case("818181818181818181", eof()),
    case("8200", eof()),
    case("a1", eof()),
    case("a20102", eof()),
    case("a100", eof()),
    case("a2000000", eof()),

    // Tag number not followed by tag content:
    case("c0", eof()),

    // Indefinite-length strings not closed by a "break" stop code:
    case("5f4100", eof()),
    case("7f6100", eof()),

    // Indefinite-length maps and arrays not closed by a "break" stop code:
    case("9f", eof()),
    case("9f0102", eof()),
    case("bf", eof()),
    case("bf01020102", eof()),
    case("819f", eof()),
    case("9f8000", eof()),
    case("9f9f9f9f9fffffffff", eof()),
    case("9f819f819f9fffffff", eof()),

    // Reserved additional information values:
    case("1c", Error::Syntax(0)),
    case("1d", Error::Syntax(0)),
    case("1e", Error::Syntax(0)),
    case("3c", Error::Syntax(0)),
    case("3d", Error::Syntax(0)),
    case("3e", Error::Syntax(0)),
    case("5c", Error::Syntax(0)),
    case("5d", Error::Syntax(0)),
    case("5e", Error::Syntax(0)),
    case("7c", Error::Syntax(0)),
    case("7d", Error::Syntax(0)),
    case("7e", Error::Syntax(0)),
    case("9c", Error::Syntax(0)),
    case("9d", Error::Syntax(0)),
    case("9e", Error::Syntax(0)),
    case("bc", Error::Syntax(0)),
    case("bd", Error::Syntax(0)),
    case("be", Error::Syntax(0)),
    case("dc", Error::Syntax(0)),
    case("dd", Error::Syntax(0)),
    case("de", Error::Syntax(0)),
    case("fc", Error::Syntax(0)),
    case("fd", Error::Syntax(0)),
    case("fe", Error::Syntax(0)),

    // Reserved two-byte encodings of simple values:
    case("f800", Error::Semantic(None, "invalid type: simple, expected known simple value".into())),
    case("f801", Error::Semantic(None, "invalid type: simple, expected known simple value".into())),
    case("f818", Error::Semantic(None, "invalid type: simple, expected known simple value".into())),
    case("f81f", Error::Semantic(None, "invalid type: simple, expected known simple value".into())),

    // Indefinite-length string chunks not of the correct type:
    case("5f00ff", Error::Syntax(1)),
    case("5f21ff", Error::Syntax(1)),
    case("5f6100ff", Error::Syntax(1)),
    case("5f80ff", Error::Syntax(1)),
    case("5fa0ff", Error::Syntax(1)),
    case("5fc000ff", Error::Syntax(1)),
    case("5fe0ff", Error::Syntax(1)),
    case("7f4100ff", Error::Syntax(1)),

    // Indefinite-length string chunks not definite length:
    //case("5f5f4100ffff", Error::Syntax(0)), These should fail, but do not currently.
    //case("7f7f6100ffff", Error::Syntax(0)),

    // Break occurring on its own outside of an indefinite-length item:
    case("ff", Error::Semantic(None, "invalid type: break, expected non-break".into())),

    // Break occurring in a definite-length array or map or a tag:
    case("81ff", Error::Semantic(None, "invalid type: break, expected non-break".into())),
    case("8200ff", Error::Semantic(None, "invalid type: break, expected non-break".into())),
    case("a1ff", Error::Semantic(None, "invalid type: break, expected non-break".into())),
    case("a1ff00", Error::Semantic(None, "invalid type: break, expected non-break".into())),
    case("a100ff", Error::Semantic(None, "invalid type: break, expected non-break".into())),
    case("a20000ff", Error::Semantic(None, "invalid type: break, expected non-break".into())),
    case("9f81ff", Error::Semantic(None, "invalid type: break, expected non-break".into())),
    case("9f829f819f9fffffffff", Error::Semantic(None, "invalid type: break, expected non-break".into())),

    // Break in an indefinite-length map that would lead to an odd number of items (break in a value position):
    case("bf00ff", Error::Semantic(None, "invalid type: break, expected non-break".into())),
    case("bf000000ff", Error::Semantic(None, "invalid type: break, expected non-break".into())),

    // Major type 0, 1, 6 with additional information 31:
    case("1f", Error::Syntax(0)),
    case("3f", Error::Syntax(0)),
    case("df", Error::Syntax(0)),
)]
fn test(bytes: &str, error: Error<std::io::Error>) {
    let bytes = hex::decode(bytes).unwrap();

    // Due to no_std support, pretend that all io errors are EOF
    let correct = match error {
        Error::Io(..) => ("io", None, None),
        Error::Syntax(x) => ("syntax", Some(x), None),
        Error::Semantic(x, y) => ("semantic", x, Some(y)),
        Error::RecursionLimitExceeded => panic!(),
    };

    let result: Result<Value, _> = from_reader(dbg!(&bytes[..]));
    let actual = match dbg!(result.unwrap_err()) {
        Error::Io(..) => ("io", None, None),
        Error::Syntax(x) => ("syntax", Some(x), None),
        Error::Semantic(x, y) => ("semantic", x, Some(y)),
        Error::RecursionLimitExceeded => panic!(),
    };

    assert_eq!(correct, actual);
}

#[test]
fn test_long_utf8_deserialization() {
    let s = (0..2000).map(|_| 'ボ').collect::<String>();
    let mut v = Vec::new();
    into_writer(&s, &mut v).unwrap();
    let _: String = from_reader(&*v).unwrap();
}
