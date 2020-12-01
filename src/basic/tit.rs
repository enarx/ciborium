// SPDX-License-Identifier: Apache-2.0

use super::*;

use core::convert::TryFrom;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Title(pub Major, pub Minor);

impl Title {
    pub const BREAK: Self = Self(Major::Other, Minor::Indeterminate);

    pub const FALSE: Self = Self(Major::Other, Minor::Immediate(Immediate(20)));
    pub const TRUE: Self = Self(Major::Other, Minor::Immediate(Immediate(21)));
    pub const NULL: Self = Self(Major::Other, Minor::Immediate(Immediate(22)));
    pub const UNDEFINED: Self = Self(Major::Other, Minor::Immediate(Immediate(23)));

    pub const TAG_BIGPOS: Self = Self(Major::Tag, Minor::Immediate(Immediate(2)));
    pub const TAG_BIGNEG: Self = Self(Major::Tag, Minor::Immediate(Immediate(3)));
}

impl TryFrom<Title> for f32 {
    type Error = InvalidError;

    #[inline]
    #[allow(clippy::float_cmp)]
    fn try_from(value: Title) -> Result<Self, Self::Error> {
        match value.0 {
            Major::Other => f32::try_from(value.1),
            _ => Err(InvalidError(())),
        }
    }
}

impl TryFrom<Title> for f64 {
    type Error = InvalidError;

    #[inline]
    fn try_from(value: Title) -> Result<Self, Self::Error> {
        match value.0 {
            Major::Other => f64::try_from(value.1),
            _ => Err(InvalidError(())),
        }
    }
}

impl TryFrom<Title> for i128 {
    type Error = InvalidError;

    #[inline]
    fn try_from(value: Title) -> Result<Self, Self::Error> {
        let x = Option::<u64>::from(value.1).ok_or(InvalidError(()))? as i128;

        match value.0 {
            Major::Positive => Ok(x),
            Major::Negative => Ok(x ^ !0),
            _ => Err(InvalidError(())),
        }
    }
}

impl TryFrom<Title> for Option<usize> {
    type Error = InvalidError;

    #[inline]
    fn try_from(value: Title) -> Result<Self, Self::Error> {
        match value.0 {
            Major::Bytes | Major::Text | Major::Array | Major::Map => Option::<u64>::from(value.1)
                .map(usize::try_from)
                .transpose()
                .or(Err(InvalidError(()))),

            _ => Err(InvalidError(())),
        }
    }
}

impl From<bool> for Title {
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            Title::TRUE
        } else {
            Title::FALSE
        }
    }
}

impl From<u8> for Title {
    #[inline]
    fn from(value: u8) -> Self {
        Self(Major::Positive, Minor::from(u64::from(value)))
    }
}

impl From<u16> for Title {
    #[inline]
    fn from(value: u16) -> Self {
        Self(Major::Positive, Minor::from(u64::from(value)))
    }
}

impl From<u32> for Title {
    #[inline]
    fn from(value: u32) -> Self {
        Self(Major::Positive, Minor::from(u64::from(value)))
    }
}

impl From<u64> for Title {
    #[inline]
    fn from(value: u64) -> Self {
        Self(Major::Positive, Minor::from(value))
    }
}

impl TryFrom<u128> for Title {
    type Error = InvalidError;

    #[inline]
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        let x = u64::try_from(value).map_err(|_| InvalidError(()))?;
        Ok(Self(Major::Positive, Minor::from(x)))
    }
}

impl From<i8> for Title {
    #[inline]
    fn from(value: i8) -> Self {
        let (major, value) = match value.is_negative() {
            false => (Major::Positive, value as u64),
            true => (Major::Negative, value as u64 ^ !0),
        };

        Self(major, Minor::from(value))
    }
}

impl From<i16> for Title {
    #[inline]
    fn from(value: i16) -> Self {
        let (major, value) = match value.is_negative() {
            false => (Major::Positive, value as u64),
            true => (Major::Negative, value as u64 ^ !0),
        };

        Self(major, Minor::from(value))
    }
}

impl From<i32> for Title {
    #[inline]
    fn from(value: i32) -> Self {
        let (major, value) = match value.is_negative() {
            false => (Major::Positive, value as u64),
            true => (Major::Negative, value as u64 ^ !0),
        };

        Self(major, Minor::from(value))
    }
}

impl From<i64> for Title {
    #[inline]
    fn from(value: i64) -> Self {
        let (major, value) = match value.is_negative() {
            false => (Major::Positive, value as u64),
            true => (Major::Negative, value as u64 ^ !0),
        };

        Self(major, Minor::from(value))
    }
}

impl TryFrom<i128> for Title {
    type Error = InvalidError;

    #[inline]
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        let (major, value) = match value.is_negative() {
            false => (Major::Positive, u64::try_from(value)),
            true => (Major::Negative, u64::try_from(value ^ !0)),
        };

        Ok(Self(major, value.map_err(|_| InvalidError(()))?.into()))
    }
}

impl From<f32> for Title {
    #[inline]
    fn from(value: f32) -> Self {
        Self(Major::Other, Minor::from(value as f64))
    }
}

impl From<f64> for Title {
    #[inline]
    fn from(value: f64) -> Self {
        Self(Major::Other, Minor::from(value))
    }
}

impl Title {
    #[inline]
    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64",))]
    pub fn from_length(major: Major, length: impl Into<Option<usize>>) -> Self {
        Self(major, length.into().map(|x| x as u64).into())
    }
}
