// Tests for from_slice with Deserialize<'de> support

use serde::Deserialize;

#[test]
fn test_from_slice_with_owned_data() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Data {
        title: String,
        count: u64,
    }

    // CBOR: {"title": "Test Data", "count": 42}
    let cbor_data = b"\xa2\x65title\x69Test Data\x65count\x18\x2a";
    let data: Data = ciborium::from_slice(cbor_data).unwrap();

    assert_eq!(data.title, "Test Data");
    assert_eq!(data.count, 42);
}

#[test]
fn test_from_slice_with_deserialize_lifetime() {
    #[derive(Deserialize, Debug)]
    struct Person<'a> {
        #[serde(borrow)]
        name: &'a str,
        age: u32,
    }

    // CBOR: {"name": "Alice", "age": 30}
    let cbor_person = b"\xa2\x64name\x65Alice\x63age\x18\x1e";

    // This demonstrates that from_slice accepts Deserialize<'de> types
    let person: Person = ciborium::from_slice(cbor_person).unwrap();
    assert_eq!(person.name, "Alice");
    assert_eq!(person.age, 30);
}

#[test]
fn test_from_slice_array() {
    // CBOR: [1, 2, 3, 4, 5]
    let cbor_array = b"\x85\x01\x02\x03\x04\x05";
    let array: Vec<u32> = ciborium::from_slice(cbor_array).unwrap();
    assert_eq!(array, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_from_slice_with_buffer() {
    use std::collections::HashMap;

    // CBOR: {"a": 1, "b": 2, "c": 3}
    let cbor_map = b"\xa3\x61a\x01\x61b\x02\x61c\x03";
    let mut buffer = [0u8; 8192];

    let map: HashMap<String, u32> =
        ciborium::de::from_slice_with_buffer(cbor_map, &mut buffer).unwrap();

    assert_eq!(map.get("a"), Some(&1));
    assert_eq!(map.get("b"), Some(&2));
    assert_eq!(map.get("c"), Some(&3));
}

#[test]
fn test_from_slice_nested_structures() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Inner {
        value: i32,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Outer {
        inner: Inner,
        list: Vec<u32>,
    }

    // CBOR: {"inner": {"value": 42}, "list": [1, 2, 3]}
    let cbor = b"\xa2\x65inner\xa1\x65value\x18\x2a\x64list\x83\x01\x02\x03";
    let outer: Outer = ciborium::from_slice(cbor).unwrap();

    assert_eq!(outer.inner.value, 42);
    assert_eq!(outer.list, vec![1, 2, 3]);
}

#[test]
fn test_from_slice_with_recursion_limit() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct SimpleStruct {
        value: u32,
    }

    // CBOR: {"value": 100}
    let cbor = b"\xa1\x65value\x18\x64";
    let data: SimpleStruct = ciborium::de::from_slice_with_recursion_limit(cbor, 256).unwrap();

    assert_eq!(data.value, 100);
}
