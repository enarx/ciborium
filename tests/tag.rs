// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use ciborium::{de::from_reader, ser::into_writer, value::Bytes, Tag};

const CBOR: &[u8] = b"\xc7\x49\x01\x00\x00\x00\x00\x00\x00\x00\x00";
const FULL: Tag<Bytes<&[u8]>> = Tag(7, Bytes::new(b"\x01\x00\x00\x00\x00\x00\x00\x00\x00"));

#[test]
fn decode() {
    // Test that we can decode the tag.
    let tag: Tag<Bytes<Vec<u8>>> = from_reader(CBOR).unwrap();
    assert_eq!(FULL.0, tag.0);
    assert_eq!(FULL.1, tag.1[..].into());
}

#[test]
fn skip() {
    // Test that we can skip the tag.
    let raw: Bytes<Vec<u8>> = from_reader(CBOR).unwrap();
    assert_eq!(FULL.1, raw[..].into());
}

#[test]
fn encode() {
    // Test that we can encode the tag.
    let mut byte = Vec::new();
    into_writer(&FULL, &mut byte).unwrap();
    assert_eq!(CBOR, byte);
}
