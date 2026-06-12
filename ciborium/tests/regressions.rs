// SPDX-License-Identifier: Apache-2.0
//! Regression tests for decoding correctness and robustness fixes.

use ciborium::de::{from_reader, from_reader_with_buffer};
use ciborium::value::{CanonicalValue, Value};
use serde::Deserialize;

/// A negative bignum whose magnitude fits in 16 bytes but exceeds `i128`
/// must be preserved as `Value::Tag(3, bytes)`, exactly like longer payloads,
/// and must round-trip.
#[test]
fn bigneg_16_byte_payload_is_preserved() {
    // tag(3) + bytes(16 x 0xff) => -2^128
    let bytes = hex::decode("c350ffffffffffffffffffffffffffffffff").unwrap();
    let val: Value = from_reader(&bytes[..]).unwrap();
    assert_eq!(val, Value::Tag(3, Box::new(Value::Bytes(vec![0xff; 16]))));

    let mut out = Vec::new();
    ciborium::ser::into_writer(&val, &mut out).unwrap();
    assert_eq!(out, bytes);

    // Typed deserialization must still reject it: it doesn't fit in i128.
    let too_large: Result<i128, _> = from_reader(&bytes[..]);
    assert!(too_large.is_err());
}

/// A scratch buffer too small to reassemble a multi-byte UTF-8 character
/// must produce an error instead of looping forever.
#[test]
fn tiny_scratch_buffer_errors_instead_of_hanging() {
    let mut buf = Vec::new();
    ciborium::ser::into_writer(&"héllo", &mut buf).unwrap();

    let mut scratch = [0u8; 1];
    let result: Result<String, _> = from_reader_with_buffer(&buf[..], &mut scratch);
    assert!(result.is_err());

    // A 4-byte scratch buffer is the documented minimum and must succeed.
    let mut scratch = [0u8; 4];
    let result: Result<String, _> = from_reader_with_buffer(&buf[..], &mut scratch);
    assert_eq!(result.unwrap(), "héllo");
}

/// Two-byte encodings of simple values below 32 are not well-formed per
/// RFC 8949 section 3.3. In particular, `f814` must not decode as `false`.
#[test]
fn two_byte_simple_values_are_rejected() {
    for input in ["f814", "f815", "f816", "f817"] {
        let bytes = hex::decode(input).unwrap();
        let result: Result<Value, _> = from_reader(&bytes[..]);
        assert!(result.is_err(), "{input} must be rejected");
    }

    // The one-byte encodings remain valid.
    assert_eq!(
        from_reader::<Value, _>(&hex::decode("f4").unwrap()[..]).unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        from_reader::<Value, _>(&hex::decode("f5").unwrap()[..]).unwrap(),
        Value::Bool(true)
    );
}

#[derive(Deserialize, Debug, PartialEq)]
enum E {
    Unit,
    Newtype(u8),
    Tuple(u8, u8),
    Struct { a: u8 },
}

/// A map-form unit variant must consume its payload (and require it to be
/// null) instead of leaving it dangling in the stream.
#[test]
fn enum_unit_variant_map_form() {
    // [{"Unit": 5}, 7]: the dangling 5 used to be read as the next element.
    let bytes = hex::decode("82a164556e69740507").unwrap();
    let result: Result<(E, u8), _> = from_reader(&bytes[..]);
    assert!(result.is_err(), "unit variant payload must be null");

    // [{"Unit": null}, 7] is accepted and stays in sync.
    let bytes = hex::decode("82a164556e6974f607").unwrap();
    let result: (E, u8) = from_reader(&bytes[..]).unwrap();
    assert_eq!(result, (E::Unit, 7));

    // The canonical encoding (bare text) still works.
    let bytes = hex::decode("8264556e697407").unwrap();
    let result: (E, u8) = from_reader(&bytes[..]).unwrap();
    assert_eq!(result, (E::Unit, 7));
}

/// A bare-text variant name must not satisfy a payload-carrying variant by
/// consuming the following (sibling) item from the stream.
#[test]
fn enum_payload_variants_reject_bare_text_form() {
    // ["Newtype", 7, 9]: "Newtype" used to consume the sibling 7 as payload.
    let bytes = hex::decode("83674e6577747970650709").unwrap();
    let result: Result<(E, u8, u8), _> = from_reader(&bytes[..]);
    assert!(
        result.is_err(),
        "bare text must not satisfy a newtype variant"
    );

    // ["Tuple", 7, 9]
    let bytes = hex::decode("83655475706c650709").unwrap();
    let result: Result<(E, u8, u8), _> = from_reader(&bytes[..]);
    assert!(
        result.is_err(),
        "bare text must not satisfy a tuple variant"
    );

    // ["Struct", 7, 9]
    let bytes = hex::decode("8366537472756374 0709".replace(' ', "")).unwrap();
    let result: Result<(E, u8, u8), _> = from_reader(&bytes[..]);
    assert!(
        result.is_err(),
        "bare text must not satisfy a struct variant"
    );

    // The map forms still round-trip.
    let bytes = hex::decode("82a1674e6577747970650507").unwrap();
    let result: (E, u8) = from_reader(&bytes[..]).unwrap();
    assert_eq!(result, (E::Newtype(5), 7));

    let bytes = hex::decode("82a1655475706c65820509 07".replace(' ', "")).unwrap();
    let result: (E, u8) = from_reader(&bytes[..]).unwrap();
    assert_eq!(result, (E::Tuple(5, 9), 7));

    let bytes = hex::decode("82a166537472756374a16161 05 07".replace(' ', "")).unwrap();
    let result: (E, u8) = from_reader(&bytes[..]).unwrap();
    assert_eq!(result, (E::Struct { a: 5 }, 7));
}

/// Canonical ordering of tagged values must follow the serialized bytes
/// (length-first), not the derived `PartialOrd` of the inner values.
#[test]
fn canonical_tag_ordering_is_bytewise() {
    // Tag(1, Bytes([])) -> c1 40 (2 bytes)
    // Tag(1, Integer(1000)) -> c1 1903e8 (4 bytes)
    let a = CanonicalValue::from(Value::Tag(1, Box::new(Value::Bytes(vec![]))));
    let b = CanonicalValue::from(Value::Tag(1, Box::new(Value::Integer(1000.into()))));
    assert!(a < b, "shorter serialization must sort first");

    // Tag(1000, Integer(0)) -> d903e8 00 (4 bytes)
    // Tag(23, Text("aaaaaaaaaa")) -> d7 6a 6161.. (12 bytes)
    let c = CanonicalValue::from(Value::Tag(1000, Box::new(Value::Integer(0.into()))));
    let d = CanonicalValue::from(Value::Tag(23, Box::new(Value::Text("aaaaaaaaaa".into()))));
    assert!(c < d, "shorter serialization must sort first");

    // Same tag, same length: lexical byte order. 0x0a (10) < 0x20 (-1).
    let e = CanonicalValue::from(Value::Tag(1, Box::new(Value::Integer(10.into()))));
    let f = CanonicalValue::from(Value::Tag(1, Box::new(Value::Integer((-1).into()))));
    assert!(e < f, "equal length keys sort in lexical byte order");
}
