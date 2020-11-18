// SPDX-License-Identifier: Apache-2.0

use crate::Float;
use super::*;

use core::convert::TryFrom;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Header {
    Bytes(Option<u64>),
    Text(Option<u64>),
    Array(Option<u64>),
    Map(Option<u64>),
    Integer(i128),
    Simple(u8),
    Float(Float),
    Tag(u64),
    Break,
}

impl Header {
    pub const FALSE: Self = Self::Simple(20);
    pub const TRUE: Self = Self::Simple(21);
    pub const NULL: Self = Self::Simple(22);
    pub const UNDEFINED: Self = Self::Simple(23);
}

macro_rules! implfrom {
    ($($v:ident($t:ty)),+ $(,)?) => {
        $(
            impl From<$t> for Header {
                #[inline]
                fn from(value: $t) -> Self {
                    Self::$v(value.into())
                }
            }
        )+
    };
}

implfrom! {
    Integer(i8),
    Integer(i16),
    Integer(i32),
    Integer(i64),
    Integer(i128),
    Integer(u8),
    Integer(u16),
    Integer(u32),
    Integer(u64),
    Float(f32),
    Float(f64),
}

impl From<bool> for Header {
    #[inline]
    fn from(value: bool) -> Self {
        match value {
            false => Header::FALSE,
            true => Header::TRUE,
        }
    }
}

impl Header {
    #[inline]
    fn convert(title: Title) -> Option<Self> {
        Some(match (title.0, title.1) {
            (Major::Positive, minor) => Header::Integer(Option::<u64>::from(minor)? as i128),
            (Major::Negative, minor) => Header::Integer(Option::<u64>::from(minor)? as i128 ^ !0),
            (Major::Bytes, minor) => Header::Bytes(minor.into()),
            (Major::Text, minor) => Header::Text(minor.into()),
            (Major::Array, minor) => Header::Array(minor.into()),
            (Major::Map, minor) => Header::Map(minor.into()),
            (Major::Tag, minor) => Header::Tag(Option::<u64>::from(minor)?),

            (Major::Other, Minor::Indeterminate) => Header::Break,
            (Major::Other, Minor::Immediate(x)) => Header::Simple(x.into()),
            (Major::Other, Minor::Subsequent1(x)) => Header::Simple(x[0]),
            (Major::Other, minor) => Header::Float(Option::from(minor)?),
        })
    }
}

impl TryFrom<Title> for Header {
    type Error = Invalid;

    #[inline]
    fn try_from(title: Title) -> Result<Self, Self::Error> {
        Header::convert(title).ok_or(Invalid(()))
    }
}

impl TryFrom<Header> for Title {
    type Error = Invalid;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        Ok(match value {
            Header::Bytes(x) => Title(Major::Bytes, x.into()),
            Header::Text(x) => Title(Major::Text, x.into()),
            Header::Array(x) => Title(Major::Array, x.into()),
            Header::Map(x) => Title(Major::Map, x.into()),
            Header::Tag(x) => Title(Major::Tag, x.into()),

            Header::Simple(x) => Title(Major::Other, u64::from(x).into()),
            Header::Float(x) => Title(Major::Other, x.into()),
            Header::Break => Title(Major::Other, Minor::Indeterminate),

            Header::Integer(x) => {
                if x < 0 {
                    let x = u64::try_from(x ^ !0).or(Err(Invalid(())))?;
                    Title(Major::Negative, x.into())
                } else {
                    let x = u64::try_from(x).or(Err(Invalid(())))?;
                    Title(Major::Positive, x.into())
                }
            }
        })
    }
}
