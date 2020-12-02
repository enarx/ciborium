// SPDX-License-Identifier: Apache-2.0

use super::*;

use core::convert::TryFrom;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Title(pub Major, pub Minor);

impl Title {
    pub const BREAK: Self = Self(Major::Other, Minor::Indeterminate);

    pub const FALSE: Self = Self(Major::Other, Minor::Immediate(20));
    pub const TRUE: Self = Self(Major::Other, Minor::Immediate(21));
    pub const NULL: Self = Self(Major::Other, Minor::Immediate(22));
    pub const UNDEFINED: Self = Self(Major::Other, Minor::Immediate(23));

    pub const TAG_BIGPOS: Self = Self(Major::Tag, Minor::Immediate(2));
    pub const TAG_BIGNEG: Self = Self(Major::Tag, Minor::Immediate(3));
}

impl TryFrom<u8> for Title {
    type Error = InvalidError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self(Major::from(value), Minor::try_from(value)?))
    }
}

impl From<Title> for u8 {
    #[inline]
    fn from(value: Title) -> Self {
        u8::from(value.0) | u8::from(value.1)
    }
}
