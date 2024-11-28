// SPDX-License-Identifier: Apache-2.0

extern crate std;

use ciborium::cbor;
use ciborium::tag::Required;
use ciborium::value::CanonicalValue;
use rand::prelude::*;
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
fn map_old() {
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

/// Use length-first ordering for keys.
#[test]
#[cfg(feature = "std")]
fn map_rfc7049() {
    use ciborium::canonical::Rfc7049;

    let mut map = BTreeMap::new();
    map.insert(cval!(false), val!(2));
    map.insert(cval!([-1]), val!(5));
    map.insert(cval!(-1), val!(1));
    map.insert(cval!(10), val!(0));
    map.insert(cval!(100), val!(3));
    map.insert(cval!([100]), val!(7));
    map.insert(cval!("z"), val!(4));
    map.insert(cval!("aa"), val!(6));

    let bytes1 = ciborium::ser::to_vec_canonical::<_, Rfc7049>(&map).unwrap();

    assert_eq!(
        hex::encode(bytes1),
        "a80a002001f402186403617a048120056261610681186407"
    );
}

/// Match [RFC 8949] deterministic ordering example.
///
/// The RFC specifies lexicographic byte ordering of serialized keys.
///
/// [RFC 8949]: https://www.rfc-editor.org/rfc/rfc8949.html#name-core-deterministic-encoding
#[test]
#[cfg(feature = "std")]
fn map_rfc8949() {
    use ciborium::canonical::Rfc8949;

    let mut map = BTreeMap::new();
    map.insert(cval!(false), val!(2));
    map.insert(cval!([-1]), val!(5));
    map.insert(cval!(-1), val!(1));
    map.insert(cval!(10), val!(0));
    map.insert(cval!(100), val!(3));
    map.insert(cval!([100]), val!(7));
    map.insert(cval!("z"), val!(4));
    map.insert(cval!("aa"), val!(6));

    let bytes1 = ciborium::ser::to_vec_canonical::<_, Rfc8949>(&map).unwrap();

    assert_eq!(
        hex::encode(bytes1),
        "a80a001864032001617a046261610681186407812005f402"
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
