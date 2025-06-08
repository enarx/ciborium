// SPDX-License-Identifier: Apache-2.0

extern crate std;

use ciborium::cbor;
use ciborium::tag::Required;
use ciborium::value::{canonical_into_writer, canonical_value, CanonicalValue, Value};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

macro_rules! cval {
    ($x:expr) => {
        CanonicalValue::from(val!($x))
    };
}

macro_rules! val {
    ($x:expr) => {
        cbor!($x).unwrap()
    };
}

#[test]
fn rfc8949_example() {
    let mut array: Vec<CanonicalValue> = vec![
        cval!(10),
        cval!(-1),
        cval!(false),
        cval!(100),
        cval!("z"),
        cval!([-1]),
        cval!("aa"),
        cval!([100]),
    ];
    let golden = array.clone();

    // Shuffle the array.
    array.shuffle(&mut rand::thread_rng());

    array.sort();

    assert_eq!(array, golden);
}

#[test]
fn map() {
    let mut map = BTreeMap::new();
    map.insert(cval!(false), val!(2));
    map.insert(cval!([-1]), val!(5));
    map.insert(cval!(-1), val!(1));
    map.insert(cval!(10), val!(0));
    map.insert(cval!(100), val!(3));
    map.insert(cval!([100]), val!(7));
    map.insert(cval!("z"), val!(4));
    map.insert(cval!("aa"), val!(6));

    let mut bytes1 = Vec::new();
    ciborium::ser::into_writer(&map, &mut bytes1).unwrap();

    assert_eq!(
        hex::encode(&bytes1),
        "a80a002001f402186403617a048120056261610681186407"
    );
}

#[test]
fn negative_numbers() {
    let mut array: Vec<CanonicalValue> = vec![
        cval!(10),
        cval!(-1),
        cval!(-2),
        cval!(-3),
        cval!(-4),
        cval!(false),
        cval!(100),
        cval!(-100),
        cval!(-200),
        cval!("z"),
        cval!([-1]),
        cval!(-300),
        cval!("aa"),
        cval!([100]),
    ];
    let golden = array.clone();

    // Shuffle the array.
    array.shuffle(&mut rand::thread_rng());

    array.sort();

    assert_eq!(array, golden);
}

#[test]
fn tagged_option() {
    let mut opt = Some(Required::<u64, 0xff>(2u32.into()));

    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&opt, &mut bytes).unwrap();

    let output = ciborium::de::from_reader(&bytes[..]).unwrap();
    assert_eq!(opt, output);

    opt = None;

    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&opt, &mut bytes).unwrap();

    let output = ciborium::de::from_reader(&bytes[..]).unwrap();
    assert_eq!(opt, output);
}

#[test]
fn canonical_value_example() {
    let map = Value::Map(vec![
        (val!(false), val!(2)),
        (val!([-1]), val!(5)),
        (val!(-1), val!(1)),
        (val!(10), val!(0)),
        (val!(100), val!(3)),
        (val!([100]), val!(7)),
        (val!("z"), val!(4)),
        (val!("aa"), val!(6)),
    ]);

    let mut bytes = Vec::new();
    canonical_into_writer(&map, &mut bytes).unwrap();
    assert_eq!(
        hex::encode(&bytes),
        "a80a002001f402186403617a048120056261610681186407"
    );

    bytes.clear();
    let canonical = canonical_value(map);
    ciborium::ser::into_writer(&canonical, &mut bytes).unwrap();

    assert_eq!(
        hex::encode(&bytes),
        "a80a002001f402186403617a048120056261610681186407"
    );
}

#[test]
fn canonical_value_nested_structures() {
    // Create nested structure with unsorted maps
    let nested = Value::Array(vec![
        Value::Map(vec![(val!("b"), val!(2)), (val!("a"), val!(1))]),
        Value::Tag(
            1,
            Box::new(Value::Map(vec![
                (val!(100), val!("high")),
                (val!(10), val!("low")),
            ])),
        ),
    ]);

    let canonical = canonical_value(nested);

    if let Value::Array(elements) = canonical {
        // Check first map is sorted
        if let Value::Map(entries) = &elements[0] {
            assert_eq!(entries[0].0, val!("a"));
            assert_eq!(entries[1].0, val!("b"));
        }

        // Check tagged map is sorted
        if let Value::Tag(_, inner) = &elements[1] {
            if let Value::Map(entries) = inner.as_ref() {
                assert_eq!(entries[0].0, val!(10));
                assert_eq!(entries[1].0, val!(100));
            }
        }
    } else {
        panic!("Expected Array value");
    }
}

#[test]
fn canonical_value_struct() {
    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct T1 {
        a: u32,
        b: u32,
        c: u32,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct T2 {
        c: u32,
        b: u32,
        a: u32,
    }

    let t1 = T1 { a: 1, b: 2, c: 3 };
    let t2 = T2 { c: 3, b: 2, a: 1 };

    let mut bytes1 = Vec::new();
    canonical_into_writer(&t1, &mut bytes1).unwrap();

    let mut bytes2 = Vec::new();
    canonical_into_writer(&t2, &mut bytes2).unwrap();
    assert_eq!(bytes1, bytes2);
}
