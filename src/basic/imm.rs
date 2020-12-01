// SPDX-License-Identifier: Apache-2.0

use super::*;

use core::convert::TryFrom;

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
                type Error = InvalidError;

                #[inline]
                fn try_from(value: $t) -> Result<Self, Self::Error> {
                    match value {
                        x @ 0..=23 => Ok(Self(x as u8)),
                        _ => Err(InvalidError(()))
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
