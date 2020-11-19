// SPDX-License-Identifier: Apache-2.0

use ciborium::basic::*;

use core::convert::TryFrom;

use rstest::rstest;

macro_rules! float {
    ($v:ident($x:expr)) => {
        Title(Major::Other, Minor::$v($x.to_be_bytes()))
    };
}

#[rstest(title, bytes,
    case(Title::from(0), "00"),
    case(Title::from(1), "01"),
    case(Title::from(10), "0a"),
    case(Title::from(23), "17"),
    case(Title::from(24), "1818"),
    case(Title::from(25), "1819"),
    case(Title::from(100), "1864"),
    case(Title::from(1000), "1903e8"),
    case(Title::from(1000000), "1a000f4240"),
    case(Title::from(1000000000000u64), "1b000000e8d4a51000"),
    case(Title::from(18446744073709551615u64), "1bffffffffffffffff"),

    case(Title::try_from(-18446744073709551616i128).unwrap(), "3bffffffffffffffff"),
    case(Title::from(-1), "20"),
    case(Title::from(-10), "29"),
    case(Title::from(-100), "3863"),
    case(Title::from(-1000), "3903e7"),

    case(Title::from(0.0), "f90000"),
    case(Title::from(-0.0), "f98000"),
    case(Title::from(1.0), "f93c00"),
    case(Title::from(1.1), "fb3ff199999999999a"),
    case(Title::from(1.5), "f93e00"),
    case(Title::from(65504.0), "f97bff"),
    case(Title::from(100000.0), "fa47c35000"),
    case(Title::from(3.4028234663852886e+38), "fa7f7fffff"),
    case(Title::from(1.0e+300), "fb7e37e43c8800759c"),
    case(Title::from(5.960464477539063e-8), "f90001"),
    case(Title::from(0.00006103515625), "f90400"),
    case(Title::from(-4.0), "f9c400"),
    case(Title::from(-4.1), "fbc010666666666666"),
    case(Title::from(core::f64::INFINITY), "f97c00"),
    case(Title::from(core::f64::NAN), "f97e00"),
    case(Title::from(-core::f64::INFINITY), "f9fc00"),
    case(float![Subsequent4(core::f32::INFINITY)], "fa7f800000"),
    case(float![Subsequent4(core::f32::NAN)], "fa7fc00000"),
    case(float![Subsequent4(-core::f32::INFINITY)], "faff800000"),
    case(float![Subsequent8(core::f64::INFINITY)], "fb7ff0000000000000"),
    case(float![Subsequent8(core::f64::NAN)], "fb7ff8000000000000"),
    case(float![Subsequent8(-core::f64::INFINITY)], "fbfff0000000000000"),

    case(Title::FALSE, "f4"),
    case(Title::TRUE, "f5"),
    case(Title::NULL, "f6"),
    case(Title::UNDEFINED, "f7"),

    case(Title(Major::Other, Minor::from(16u64)), "f0"),
    case(Title(Major::Other, Minor::from(24u64)), "f818"),
    case(Title(Major::Other, Minor::from(255u64)), "f8ff"),

    case(Title(Major::Tag, Minor::from(0u64)), "c0"),
    case(Title(Major::Tag, Minor::from(1u64)), "c1"),
    case(Title(Major::Tag, Minor::from(23u64)), "d7"),
    case(Title(Major::Tag, Minor::from(24u64)), "d818"),
    case(Title(Major::Tag, Minor::from(32u64)), "d820"),

    case(Title(Major::Bytes, Minor::from(0u64)), "40"),
    case(Title(Major::Bytes, Minor::from(1u64)), "41"),
    case(Title(Major::Bytes, Minor::from(23u64)), "57"),
    case(Title(Major::Bytes, Minor::Indeterminate), "5f"),
    case(Title(Major::Bytes, Minor::from(24u64)), "5818"),
    case(Title(Major::Bytes, Minor::from(32u64)), "5820"),

    case(Title(Major::Text, Minor::from(0u64)), "60"),
    case(Title(Major::Text, Minor::from(1u64)), "61"),
    case(Title(Major::Text, Minor::from(23u64)), "77"),
    case(Title(Major::Text, Minor::Indeterminate), "7f"),
    case(Title(Major::Text, Minor::from(24u64)), "7818"),
    case(Title(Major::Text, Minor::from(32u64)), "7820"),

    case(Title(Major::Array, Minor::from(0u64)), "80"),
    case(Title(Major::Array, Minor::from(1u64)), "81"),
    case(Title(Major::Array, Minor::from(23u64)), "97"),
    case(Title(Major::Array, Minor::Indeterminate), "9f"),
    case(Title(Major::Array, Minor::from(24u64)), "9818"),
    case(Title(Major::Array, Minor::from(32u64)), "9820"),

    case(Title(Major::Map, Minor::from(0u64)), "a0"),
    case(Title(Major::Map, Minor::from(1u64)), "a1"),
    case(Title(Major::Map, Minor::from(23u64)), "b7"),
    case(Title(Major::Map, Minor::Indeterminate), "bf"),
    case(Title(Major::Map, Minor::from(24u64)), "b818"),
    case(Title(Major::Map, Minor::from(32u64)), "b820"),
)]
fn test(title: Title, bytes: &str) {
    eprintln!("{:?}", title);

    let bytes = hex::decode(bytes).unwrap();

    // encode
    let mut buffer = Vec::new();
    title.encode(&mut buffer).unwrap();
    assert_eq!(bytes, &buffer[..]);

    // decode
    assert_eq!(title, Title::decode(&mut &bytes[..]).unwrap());
}
