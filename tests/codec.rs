// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use ciborium::value::Value;
use ciborium::{cbor, de::from_reader, ser::into_writer};

use rstest::rstest;

#[rstest(value, bytes, alternate,
    ////// The following test are values from the RFC
    case(cbor!(0).unwrap(), "00", false),
    case(cbor!(1).unwrap(), "01", false),
    case(cbor!(1).unwrap(), "1b0000000000000001", true),
    case(cbor!(10).unwrap(), "0a", false),
    case(cbor!(23).unwrap(), "17", false),
    case(cbor!(24).unwrap(), "1818", false),
    case(cbor!(25).unwrap(), "1819", false),
    case(cbor!(100).unwrap(), "1864", false),
    case(cbor!(1000).unwrap(), "1903e8", false),
    case(cbor!(1000000).unwrap(), "1a000f4240", false), // 10
    case(cbor!(1000000000000u128).unwrap(), "1b000000e8d4a51000", false),
    case(cbor!(18446744073709551615u128).unwrap(), "1bffffffffffffffff", false),
    case(cbor!(18446744073709551616u128).unwrap(), "c249010000000000000000", false),

    case(cbor!(-18446744073709551616i128).unwrap(), "3bffffffffffffffff", false),
    case(cbor!(-18446744073709551617i128).unwrap(), "c349010000000000000000", false),
    case(cbor!(-1).unwrap(), "20", false),
    case(cbor!(-1).unwrap(), "3b0000000000000000", true),
    case(cbor!(-10).unwrap(), "29", false),
    case(cbor!(-100).unwrap(), "3863", false),
    case(cbor!(-1000).unwrap(), "3903e7", false), // 20

    case(cbor!(0.0).unwrap(), "f90000", false),
    case(cbor!(-0.0).unwrap(), "f98000", false),
    case(cbor!(1.0).unwrap(), "f93c00", false),
    case(cbor!(1.1).unwrap(), "fb3ff199999999999a", false),
    case(cbor!(1.5).unwrap(), "f93e00", false),
    case(cbor!(65504.0).unwrap(), "f97bff", false),
    case(cbor!(100000.0).unwrap(), "fa47c35000", false),
    case(cbor!(3.4028234663852886e+38).unwrap(), "fa7f7fffff", false),
    case(cbor!(1.0e+300).unwrap(), "fb7e37e43c8800759c", false),
    case(cbor!(5.960464477539063e-8).unwrap(), "f90001", false), // 30
    case(cbor!(0.00006103515625).unwrap(), "f90400", false),
    case(cbor!(-4.0).unwrap(), "f9c400", false),
    case(cbor!(-4.1).unwrap(), "fbc010666666666666", false),
    case(cbor!(core::f64::INFINITY).unwrap(), "f97c00", false),
    case(cbor!(core::f64::NAN).unwrap(), "f97e00", false),
    case(cbor!(-core::f64::INFINITY).unwrap(), "f9fc00", false),
    case(cbor!(core::f64::INFINITY).unwrap(), "fa7f800000", true),
    case(cbor!(core::f64::NAN).unwrap(), "fa7fc00000", true),
    case(cbor!(-core::f64::INFINITY).unwrap(), "faff800000", true),
    case(cbor!(core::f64::INFINITY).unwrap(), "fb7ff0000000000000", true), // 40
    case(cbor!(core::f64::NAN).unwrap(), "fb7ff8000000000000", true),
    case(cbor!(-core::f64::INFINITY).unwrap(), "fbfff0000000000000", true),

    case(cbor!(false).unwrap(), "f4", false),
    case(cbor!(true).unwrap(), "f5", false),
    case(cbor!(null).unwrap(), "f6", false),

    case(cbor!(Value::from(&b""[..])).unwrap(), "40", false),
    case(cbor!(Value::from(&b"\x01\x02\x03\x04"[..])).unwrap(), "4401020304", false),
    case(cbor!(Value::from(&b"\x01\x02\x03\x04\x05"[..])).unwrap(), "5f42010243030405ff", true),

    case(cbor!("").unwrap(), "60", false),
    case(cbor!("a").unwrap(), "6161", false), // 50
    case(cbor!("IETF").unwrap(), "6449455446", false),
    case(cbor!("\"\\").unwrap(), "62225c", false),
    case(cbor!("ü").unwrap(), "62c3bc", false),
    case(cbor!("水").unwrap(), "63e6b0b4", false),
    case(cbor!("𐅑").unwrap(), "64f0908591", false),
    case(cbor!("streaming").unwrap(), "7f657374726561646d696e67ff", true),

    case(cbor!([]).unwrap(), "80", false),
    case(cbor!([1, 2, 3]).unwrap(), "83010203", false),
    case(cbor!([1, [2, 3], [4, 5]]).unwrap(), "8301820203820405", false),
    case(cbor!([
        1, 2, 3, 4, 5, 6, 7, 8, 9,
        10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23,
        24, 25
    ]).unwrap(), "98190102030405060708090a0b0c0d0e0f101112131415161718181819", false), // 60
    case(cbor!([]).unwrap(), "9fff", true),
    case(cbor!([1, [2, 3], [4, 5]]).unwrap(), "9f018202039f0405ffff", true),
    case(cbor!([1, [2, 3], [4, 5]]).unwrap(), "9f01820203820405ff", true),
    case(cbor!([1, [2, 3], [4, 5]]).unwrap(), "83018202039f0405ff", true),
    case(cbor!([1, [2, 3], [4, 5]]).unwrap(), "83019f0203ff820405", true),
    case(cbor!([
        1, 2, 3, 4, 5, 6, 7, 8, 9,
        10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23,
        24, 25
    ]).unwrap(), "9f0102030405060708090a0b0c0d0e0f101112131415161718181819ff", true),

    case(cbor!({}).unwrap(), "a0", false),
    case(cbor!({1 => 2, 3 => 4}).unwrap(), "a201020304", false),
    case(cbor!({"a" => 1, "b" => [2, 3]}).unwrap(), "a26161016162820203", false),
    case(cbor!(["a", {"b" => "c"}]).unwrap(), "826161a161626163", false), // 70
    case(cbor!({
        "a" => "A",
        "b" => "B",
        "c" => "C",
        "d" => "D",
        "e" => "E"
    }).unwrap(), "a56161614161626142616361436164614461656145", false),
    case(cbor!({"a" => 1, "b" => [2, 3]}).unwrap(), "bf61610161629f0203ffff", true),
    case(cbor!(["a", {"b" => "c"}]).unwrap(), "826161bf61626163ff", true),
    case(cbor!({"Fun" => true, "Amt" => -2}).unwrap(), "bf6346756ef563416d7421ff", true),

    ////// The following test are NOT values from the RFC
    // Test that we can decode BigNums with leading zeroes (see RFC section 2.4.2)
    case(cbor!(1u8).unwrap(), "C2540000000000000000000000000000000000000001", true),
)]
fn test(value: Value, bytes: &str, alternate: bool) {
    let bytes = hex::decode(bytes).unwrap();

    if !alternate {
        let mut encoded = Vec::new();
        into_writer(&value, &mut encoded).unwrap();
        assert_eq!(bytes, encoded);
    }

    let decoded: Value = from_reader(&bytes[..]).unwrap();
    assert_eq!(value, decoded);
}
