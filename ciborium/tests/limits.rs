// SPDX-License-Identifier: Apache-2.0

//! Tests for configurable CBOR item limits during deserialization.

use ciborium::{
    de::{
        deserializer_from_reader_with_buffer,
        deserializer_from_reader_with_buffer_and_recursion_limit, from_reader, Error,
    },
    into_writer,
    value::Value,
};
use serde::Deserialize;

fn decode_with_max_items(
    bytes: &[u8],
    max_items: usize,
) -> Result<Value, Error<<&[u8] as ciborium_io::Read>::Error>> {
    let mut scratch = [0u8; 4096];
    let mut de =
        deserializer_from_reader_with_buffer(bytes, &mut scratch).with_max_items(max_items);
    Value::deserialize(&mut de)
}

fn assert_item_limit_exceeded<T: core::fmt::Debug>(res: Result<Value, Error<T>>) {
    match res {
        Err(Error::Semantic(_, msg)) if msg.contains("item limit exceeded") => {}
        other => panic!("expected item limit exceeded error, got: {other:?}"),
    }
}

#[test]
fn definite_array_item_limit() {
    // 1 outer array + 1023 nulls = 1024 CBOR items.
    let val = Value::Array(vec![Value::Null; 1023]);
    let mut buf = Vec::new();
    into_writer(&val, &mut buf).unwrap();

    assert_eq!(decode_with_max_items(&buf, 1024).unwrap(), val);
    assert_item_limit_exceeded(decode_with_max_items(&buf, 1023));

    // Default deserializer remains unlimited for backwards compatibility.
    let larger = Value::Array(vec![Value::Null; 1025]);
    buf.clear();
    into_writer(&larger, &mut buf).unwrap();
    assert_eq!(from_reader::<Value, _>(buf.as_slice()).unwrap(), larger);
}

#[test]
fn indefinite_array_item_limit() {
    // 1 outer indefinite array + 1023 nulls = 1024 items.
    let mut buf = vec![0x9f];
    buf.resize(1 + 1023, 0xf6);
    buf.push(0xff);

    assert!(decode_with_max_items(&buf, 1024).is_ok());
    assert_item_limit_exceeded(decode_with_max_items(&buf, 1023));

    // Large flat indefinite array (4094 nulls).
    let mut flood = vec![0x9f];
    flood.resize(1 + 4094, 0xf6);
    flood.push(0xff);
    assert_item_limit_exceeded(decode_with_max_items(&flood, 512));
}

#[test]
fn definite_and_indefinite_map_item_limit() {
    // 1 outer map + 10 key/value pairs = 21 items.
    let pairs: Vec<(Value, Value)> = (0u64..10)
        .map(|i| (Value::Integer(i.into()), Value::Null))
        .collect();
    let val = Value::Map(pairs);
    let mut def_buf = Vec::new();
    into_writer(&val, &mut def_buf).unwrap();

    assert_eq!(decode_with_max_items(&def_buf, 21).unwrap(), val);
    assert_item_limit_exceeded(decode_with_max_items(&def_buf, 20));

    // Indefinite map with 10 key/value pairs (0xbf ... 0xff) = 21 items.
    let mut indef_buf = vec![0xbf];
    for i in 0u8..10 {
        indef_buf.push(i);
        indef_buf.push(0xf6);
    }
    indef_buf.push(0xff);

    assert_eq!(decode_with_max_items(&indef_buf, 21).unwrap(), val);
    assert_item_limit_exceeded(decode_with_max_items(&indef_buf, 20));

    // Truncated definite map claiming 1_000_000 entries fails immediately on item limit.
    let huge_header = [0xba, 0x00, 0x0f, 0x42, 0x40];
    assert_item_limit_exceeded(decode_with_max_items(&huge_header, 1024));
}

#[test]
fn nested_fanout_item_limit() {
    // 1 outer array + 10 inner arrays + 100 nulls = 111 items at depth 2.
    let inner = Value::Array(vec![Value::Null; 10]);
    let val = Value::Array(vec![inner; 10]);
    let mut buf = Vec::new();
    into_writer(&val, &mut buf).unwrap();

    assert_eq!(decode_with_max_items(&buf, 111).unwrap(), val);
    assert_item_limit_exceeded(decode_with_max_items(&buf, 110));
}

#[test]
fn indefinite_bstr_chunks_item_limit() {
    // 1 indefinite bstr header + 20 empty chunks = 21 items.
    let mut buf = vec![0x5f];
    buf.resize(1 + 20, 0x40);
    buf.push(0xff);

    assert_eq!(
        decode_with_max_items(&buf, 21).unwrap(),
        Value::Bytes(Vec::new())
    );
    assert_item_limit_exceeded(decode_with_max_items(&buf, 20));
}

#[test]
fn recursion_and_item_limits_compose() {
    let mut scratch = [0u8; 4096];

    // 17-deep nested array exceeds recursion limit 16 even though item count (18) < 32.
    let mut deep = vec![0x81; 17];
    deep.push(0x00);
    let mut de =
        deserializer_from_reader_with_buffer_and_recursion_limit(deep.as_slice(), &mut scratch, 16)
            .with_max_items(32);
    match Value::deserialize(&mut de).unwrap_err() {
        Error::RecursionLimitExceeded => {}
        other => panic!("expected RecursionLimitExceeded, got: {other:?}"),
    }

    // Flat array of 32 nulls = 33 items exceeds max_items 32 even though depth (1) <= 16.
    let mut wide = vec![0x9f];
    wide.resize(1 + 32, 0xf6);
    wide.push(0xff);
    let mut de =
        deserializer_from_reader_with_buffer_and_recursion_limit(wide.as_slice(), &mut scratch, 16)
            .with_max_items(32);
    assert_item_limit_exceeded(Value::deserialize(&mut de));
}
