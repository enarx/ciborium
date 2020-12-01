// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::value::Float;

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

impl TryFrom<Title> for Float {
    type Error = InvalidError;

    #[inline]
    fn try_from(value: Title) -> Result<Self, Self::Error> {
        Ok(match (value.0, value.1) {
            (Major::Other, Minor::Subsequent2(x)) => half::f16::from_be_bytes(x).into(),
            (Major::Other, Minor::Subsequent4(x)) => f32::from_be_bytes(x).into(),
            (Major::Other, Minor::Subsequent8(x)) => f64::from_be_bytes(x).into(),
            _ => return Err(InvalidError(())),
        })
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

impl Title {
    #[inline]
    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64",))]
    pub fn from_length(major: Major, length: impl Into<Option<usize>>) -> Self {
        Self(major, length.into().map(|x| x as u64).into())
    }
}
