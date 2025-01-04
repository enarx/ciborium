// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use ciborium::{de::from_reader, ser::into_writer, simple_type::SimpleType, value::Value};
use rstest::rstest;
use serde::{de::DeserializeOwned, Serialize};

use core::fmt::Debug;

#[rstest(item, bytes, value, encode, success,
      case(SimpleType(0), "e0", Value::Simple(0), true, false), // Registered via Standard Actions
      case(SimpleType(19), "f3", Value::Simple(19), true, false), // Registered via Standard Actions
      case(SimpleType(23), "f7", Value::Simple(23), true, true), // CBOR simple value "undefined"
      case(SimpleType(32), "f820", Value::Simple(32), true, true),
      case(SimpleType(255), "f8ff", Value::Simple(255), true, true),
      case(vec![SimpleType(255)], "81f8ff", Value::Array(vec![Value::Simple(255)]), true, true),
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
