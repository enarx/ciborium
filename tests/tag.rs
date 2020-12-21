// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use ciborium::{de::from_reader, ser::into_writer, value::Bytes, Tag, value::Value};

const CBOR: &[u8] = b"\xc7\x49\x01\x00\x00\x00\x00\x00\x00\x00\x00";
const FULL: Tag<Bytes<&[u8]>> = Tag(7, Bytes::new(b"\x01\x00\x00\x00\x00\x00\x00\x00\x00"));

#[test]
// Test that we can decode the tag.
fn decode() {
    let tag: Tag<Bytes<Vec<u8>>> = from_reader(CBOR).unwrap();
    assert_eq!(FULL.0, tag.0);
    assert_eq!(FULL.1, tag.1[..].into());

    let value = Value::Bytes(CBOR[2..].into());
    let value = Value::Tag(7, value.into());

    let tag: Tag<Bytes<Vec<u8>>> = value.deserialized().unwrap();
    assert_eq!(FULL.0, tag.0);
    assert_eq!(FULL.1, tag.1[..].into());
}

// Test that we can skip the tag.
#[test]
fn skip() {
    let raw: Bytes<Vec<u8>> = from_reader(CBOR).unwrap();
    assert_eq!(FULL.1, raw[..].into());

    let value = Value::Bytes(CBOR[2..].into());
    let value = Value::Tag(7, value.into());

    let raw: Bytes<Vec<u8>> = value.deserialized().unwrap();
    assert_eq!(FULL.1, raw[..].into());
}

// Test that we can encode the tag.
#[test]
fn encode() {
    let mut byte = Vec::new();
    into_writer(&FULL, &mut byte).unwrap();
    assert_eq!(CBOR, byte);

    let value = Value::Bytes(CBOR[2..].into());
    let value = Value::Tag(7, value.into());

    assert_eq!(value, Value::serialized(&FULL).unwrap());
}
