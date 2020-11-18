// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "serde")]

extern crate alloc;

use ciborium::{
    cbor,
    de::from_reader,
    ser::into_writer,
    value::{Bytes, Value},
};

use alloc::collections::BTreeMap;
use core::fmt::Debug;

use rstest::rstest;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Eq)]
struct UnitStruct;

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Eq)]
struct TupleStruct(u8, u16);

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Eq)]
struct Newtype(u8);

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Eq)]
enum Enum {
    Unit,
    Newtype(u8),
    Tuple(u8, u16),
    Struct { first: u8, second: u16 },
}

#[rstest(item, value,
    case(true, cbor!(true).unwrap()),
    case(false, cbor!(false).unwrap()),

    case(7u8, cbor!(7).unwrap()),
    case(7u16, cbor!(7).unwrap()),
    case(7u32, cbor!(7).unwrap()),
    case(7u64, cbor!(7).unwrap()),
    case(7u128, cbor!(7).unwrap()),

    case(7i8, cbor!(7).unwrap()),
    case(-7i8, cbor!(-7).unwrap()),
    case(7i16, cbor!(7).unwrap()),
    case(-7i16, cbor!(-7).unwrap()),
    case(7i32, cbor!(7).unwrap()),
    case(-7i32, cbor!(-7).unwrap()),
    case(7i64, cbor!(7).unwrap()),
    case(-7i64, cbor!(-7).unwrap()),
    case(7i128, cbor!(7).unwrap()),
    case(-7i128, cbor!(-7).unwrap()),

    case('e', cbor!('e').unwrap()),
    case('é', cbor!('é').unwrap()),
    case("foo".to_string(), cbor!("foo").unwrap()),

    case(Bytes::from(&b"\x00\x01\x02\x03"[..]), Value::Bytes(vec![0, 1, 2, 3].into())),

    case(Option::<u8>::None, cbor!(null).unwrap()),
    case(Option::<u8>::Some(7), cbor!(7).unwrap()),

    case((), cbor!(null).unwrap()),
    case(UnitStruct, cbor!(null).unwrap()),
    case(Newtype(123), cbor!(123).unwrap()),

    case(vec![1usize, 2, 3], cbor!([1, 2, 3]).unwrap()),
    case((22u8, 23u16), cbor!([22, 23]).unwrap()),
    case(TupleStruct(33, 34), cbor!([33, 34]).unwrap()),

    case({
        let mut map = BTreeMap::new();
        map.insert("foo".to_string(), "bar".to_string());
        map.insert("baz".to_string(), "qux".to_string());
        map
    }, cbor!({"baz" => "qux", "foo" => "bar"}).unwrap()),

    case(Enum::Unit, cbor!({"Unit" => null}).unwrap()),
    case(Enum::Newtype(45), cbor!({"Newtype" => 45}).unwrap()),
    case(Enum::Tuple(56, 67), cbor!({"Tuple"=> vec![56, 67]}).unwrap()),
    case(Enum::Struct {
        first: 78,
        second: 89
    }, cbor!({
        "Struct" => {
            "first" => 78,
            "second" => 89
        }
    }).unwrap()),
)]
fn test<'de, T: Serialize + Deserialize<'de> + Debug + Eq>(item: T, value: Value) {
    // Encoded into/from CBOR
    let mut buf = Vec::new();
    into_writer(&item, &mut buf).unwrap();
    eprintln!("{}", hex::encode(&buf));
    let back: T = from_reader(&buf[..]).unwrap();
    assert_eq!(item, back);

    // Encoded into/from ciborium::serde::value::Value
    let val = Value::serialized(&item).unwrap();
    eprintln!("{:?}", val);
    assert_eq!(value, val);
    let back: T = val.deserialized().unwrap();
    assert_eq!(item, back);
}
