// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use ciborium_serde::{
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
    UnitVariant,
    NewtypeVariant(u8),
    TupleVariant(u8, u16),
    StructVariant { first: u8, second: u16 },
}

#[rstest(item,
    case(true),
    case(false),

    case(7u8),
    case(7u16),
    case(7u32),
    case(7u64),
    case(7u128),

    case(7i8),
    case(-7i8),
    case(7i16),
    case(-7i16),
    case(7i32),
    case(-7i32),
    case(7i64),
    case(-7i64),
    case(7i128),
    case(-7i128),

    case('e'),
    case('é'),
    case("foo".to_string()),

    case(Bytes::from(&b"\x00\x01\x02\x03"[..])),

    case(Option::<u8>::None),
    case(Option::<u8>::Some(7)),

    case(()),
    case(UnitStruct),
    case(Newtype(123)),

    case(vec![1usize, 2, 3]),
    case((22u8, 23u16)),
    case(TupleStruct(33, 34)),

    case({
        let mut map = BTreeMap::new();
        map.insert("foo".to_string(), "bar".to_string());
        map.insert("baz".to_string(), "qux".to_string());
        map
    }),

    case(Enum::UnitVariant),
    case(Enum::NewtypeVariant(45)),
    case(Enum::TupleVariant(56, 67)),
    case(Enum::StructVariant { first: 78, second: 89 }),
)]
fn test<'de, T: Serialize + Deserialize<'de> + Debug + Eq>(item: T) {
    // Encoded into/from CBOR
    let mut buf = Vec::new();
    into_writer(&item, &mut buf).unwrap();
    eprintln!("{}", hex::encode(&buf));
    let back: T = from_reader(&buf[..]).unwrap();
    assert_eq!(item, back);

    // Encoded into/from ciborium::serde::value::Value
    let val = Value::serialized(&item).unwrap();
    eprintln!("{:?}", val);
    let back: T = val.deserialized().unwrap();
    assert_eq!(item, back);
}
