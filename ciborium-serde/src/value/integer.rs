// SPDX-License-Identifier: Apache-2.0

macro_rules! unsigned {
    (@into usize) => {
        impl From<usize> for Integer {
            #[inline]
            fn from(value: usize) -> Self {
                #[cfg(target_pointer_width = "32")]
                let x = value as u32;

                #[cfg(target_pointer_width = "64")]
                let x = value as u64;

                #[cfg(target_pointer_width = "128")]
                let x = value as u128;

                x.into()
            }
        }
    };

    (@into $t:ident) => {
        impl From<$t> for Integer {
            #[inline]
            fn from(value: $t) -> Self {
                Self(true, value.into())
            }
        }
    };

    (@from $t:ident) => {
        impl core::convert::TryFrom<Integer> for $t {
            type Error = core::num::TryFromIntError;

            #[inline]
            fn try_from(value: Integer) -> Result<Self, Self::Error> {
                if value.0 {
                    if let Ok(x) = Self::try_from(value.1) {
                        return Ok(x)
                    }
                }

                Self::try_from(i128::min_value())
            }
        }
    };

    ($($t:ident)+) => {
        $(
            unsigned! { @into $t }
            unsigned! { @from $t }
        )+
    };
}

macro_rules! signed {
    (@into isize) => {
        impl From<isize> for Integer {
            #[inline]
            fn from(value: isize) -> Self {
                #[cfg(target_pointer_width = "32")]
                let x = value as i32;

                #[cfg(target_pointer_width = "64")]
                let x = value as i64;

                #[cfg(target_pointer_width = "128")]
                let x = value as i128;

                x.into()
            }
        }
    };

    (@into $t:ident) => {
        impl From<$t> for Integer {
            #[inline]
            fn from(value: $t) -> Self {
                Self(!value.is_negative(), value as i128 as u128)
            }
        }
    };

    (@from $t:ident) => {
        impl core::convert::TryFrom<Integer> for $t {
            type Error = core::num::TryFromIntError;

            #[inline]
            fn try_from(value: Integer) -> Result<Self, Self::Error> {
                let unsigned = match value.0 {
                    true => value.1,
                    false => match Self::try_from(value.1 as i128) {
                        Ok(x) => return Ok(x),
                        Err(_) => u128::max_value(),
                    }
                };

                Self::try_from(unsigned)
            }
        }
    };

    ($($t:ident)+) => {
        $(
            signed! { @into $t }
            signed! { @from $t }
        )+
    };
}

/// An abstract integer value
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Integer(bool, u128);

unsigned! { u8 u16 u32 u64 u128 usize }
signed! { i8 i16 i32 i64 i128 isize }
