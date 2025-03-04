// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use ciborium::{de::from_reader, ser::into_writer, simple_type::SimpleType, value::Value};
use rstest::rstest;
use serde::{de::DeserializeOwned, Serialize};

use core::fmt::Debug;
use std::collections::HashMap;

#[rstest(item, bytes, value, encode, success,
      case(SimpleType(0), "e0", Value::Simple(0), true, false), // Registered via Standard Actions
      case(SimpleType(19), "f3", Value::Simple(19), true, false), // Registered via Standard Actions
      case(SimpleType(23), "f7", Value::Simple(23), true, true), // CBOR simple value "undefined"
      case(SimpleType(32), "f820", Value::Simple(32), true, true),
      case(SimpleType(59), "f83b", Value::Simple(59), true, true),
      case(SimpleType(255), "f8ff", Value::Simple(255), true, true),
      case(vec![SimpleType(255)], "81f8ff", Value::Array(vec![Value::Simple(255)]), true, true),
      case(HashMap::<SimpleType, u8>::from_iter([(SimpleType(59), 0)]), "a1f83b00", Value::Map(vec![(Value::Simple(59), Value::Integer(0.into()))]), true, true),
)]
fn test<T: Serialize + DeserializeOwned + Debug + Eq>(
    item: T,
    bytes: &str,
    value: Value,
    encode: bool,
    success: bool,
) {
    let bytes = hex::decode(bytes).unwrap();

    if encode {
        // Encode into bytes
        let mut encoded = Vec::new();
        into_writer(&item, &mut encoded).unwrap();
        assert_eq!(bytes, encoded);

        // Encode into value
        assert_eq!(value, Value::serialized(&item).unwrap());
    }

    // Decode from bytes
    match from_reader(&bytes[..]) {
        Ok(x) if success => assert_eq!(item, x),
        Ok(..) => panic!("unexpected success"),
        Err(e) if success => panic!("{:?}", e),
        Err(..) => (),
    }

    // Decode from value
    match value.deserialized() {
        Ok(x) if success => assert_eq!(item, x),
        Ok(..) => panic!("unexpected success"),
        Err(e) if success => panic!("{:?}", e),
        Err(..) => (),
    }
}

#[test]
fn value_serialized() {
    let st = Value::Simple(59);
    assert_eq!(st.clone(), Value::serialized(&st).unwrap());

    let map_as_key = Value::Map(vec![(st.clone(), Value::Integer(0.into()))]);
    assert_eq!(map_as_key, Value::serialized(&map_as_key).unwrap());

    let map_as_value = Value::Map(vec![(Value::Integer(0.into()), st.clone())]);
    assert_eq!(map_as_value, Value::serialized(&map_as_value).unwrap());

    let array = Value::Array(vec![st]);
    assert_eq!(array, Value::serialized(&array).unwrap());
}

#[test]
fn value_deserialize() {
    let st = Value::Simple(59);

    let in_map_as_label = Value::Map(vec![(st.clone(), Value::Integer(0.into()))]);
    Value::deserialized::<Value>(&in_map_as_label).unwrap();

    let in_map_as_value = Value::Map(vec![(Value::Integer(0.into()), st.clone())]);
    Value::deserialized::<Value>(&in_map_as_value).unwrap();

    let in_array = Value::Array(vec![st.clone()]);
    Value::deserialized::<Value>(&in_array).unwrap();
}

#[test]
fn should_roundtrip() {
    let st = Value::Simple(59);

    let map = Value::Map(vec![(st.clone(), Value::Array(vec![]))]);

    let mut encoded = vec![];
    into_writer(&map, &mut encoded).unwrap();

    let decoded = from_reader::<Value, _>(&encoded[..]).unwrap();
    assert_eq!(decoded, map);
}
