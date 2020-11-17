// SPDX-License-Identifier: Apache-2.0

use super::*;

use core::convert::TryFrom;

/// An opaque type representing a value embedded in the `Minor`
///
/// The `Prefix` byte has a limited number of values that it can hold directly
/// without need for an `Affix` section. This type represents such numbers.
///
/// Although this type is opaque, it can be converted into the standard Rust
/// numeric types. Likewise, you can fallibly convert from the standard Rust
/// numeric types into an `Immediate`.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Immediate(pub(crate) u8);

macro_rules! mkfrom {
    ($($t:ty)+) => {
        $(
            impl From<Immediate> for $t {
                #[inline]
                fn from(value: Immediate) -> Self {
                    value.0 as _
                }
            }

            impl TryFrom<$t> for Immediate {
                type Error = Invalid;

                #[inline]
                fn try_from(value: $t) -> Result<Self, Self::Error> {
                    match value {
                        x @ 0..=23 => Ok(Self(x as u8)),
                        _ => Err(Invalid(()))
                    }
                }
            }
        )+
    };
}

mkfrom! {
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
}
