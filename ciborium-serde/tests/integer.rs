// SPDX-License-Identifier: Apache-2.0

use ciborium_serde::value::Integer;

use core::convert::{TryFrom, TryInto};
use core::fmt::Debug;
use core::num::TryFromIntError;

use rstest::rstest;

#[rstest(
    value,
    case(0u8),
    case(u8::min_value()),
    case(u8::max_value()),
    case(0u16),
    case(u16::min_value()),
    case(u16::max_value()),
    case(0u32),
    case(u32::min_value()),
    case(u32::max_value()),
    case(0u64),
    case(u64::min_value()),
    case(u64::max_value()),
    case(0u128),
    case(u128::min_value()),
    case(u128::max_value()),
    case(0i8),
    case(i8::min_value()),
    case(i8::max_value()),
    case(0i16),
    case(i16::min_value()),
    case(i16::max_value()),
    case(0i32),
    case(i32::min_value()),
    case(i32::max_value()),
    case(0i64),
    case(i64::min_value()),
    case(i64::max_value()),
    case(0i128),
    case(i128::min_value()),
    case(i128::max_value())
)]
fn convert(
    value: impl Into<Integer> + TryFrom<Integer, Error = TryFromIntError> + Debug + Eq + Copy,
) {
    assert_eq!(value, value.into().try_into().unwrap());
}

#[rstest(lhs, rhs,
    case(3u8, 7u16),
    case(3u8, 7i16),
    case(3i8, 7u16),

    case(-7i32, 0u64),
    case(-7i64, 7u128),
    case(-7i128, -3i8),
)]
fn ord(lhs: impl Into<Integer>, rhs: impl Into<Integer>) {
    assert!(lhs.into() < rhs.into());
}

#[rstest(
    lhs => [3u8, 3u16, 3u32, 3u64, 3u128, 3i8, 3i16, 3i32, 3i64, 3i128],
    rhs => [3u8, 3u16, 3u32, 3u64, 3u128, 3i8, 3i16, 3i32, 3i64, 3i128],
)]
fn peq(lhs: impl Into<Integer>, rhs: impl Into<Integer>) {
    assert_eq!(lhs.into(), rhs.into());
}

#[rstest(
    lhs => [-3i8, -3i16, -3i32, -3i64, -3i128],
    rhs => [-3i8, -3i16, -3i32, -3i64, -3i128],
)]
fn neq(lhs: impl Into<Integer>, rhs: impl Into<Integer>) {
    assert_eq!(lhs.into(), rhs.into());
}
