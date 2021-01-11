// SPDX-License-Identifier: Apache-2.0

//! A dynamic CBOR value

mod float;
mod integer;

mod de;
mod error;
mod ser;

pub use error::Error;
pub use float::{Float, TryFromFloatError};
pub use integer::Integer;

use alloc::{boxed::Box, string::String, vec::Vec};
use core::convert::TryFrom;

/// A representation of a dynamic CBOR value that can handled dynamically
#[non_exhaustive]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    /// An integer
    Integer(Integer),

    /// Bytes
    Bytes(Vec<u8>),

    /// A float
    Float(Float),

    /// A string
    Text(String),

    /// A boolean
    Bool(bool),

    /// Null
    Null,

    /// Tag
    Tag(u64, Box<Value>),

    /// An array
    Array(Vec<Value>),

    /// A map
    Map(Vec<(Value, Value)>),
}

macro_rules! implfrom {
    ($($v:ident($t:ty)),+ $(,)?) => {
        $(
            impl From<$t> for Value {
                #[inline]
                fn from(value: $t) -> Self {
                    Self::$v(value.into())
                }
            }
        )+
    };
}

implfrom! {
    Integer(Integer),
    Integer(u64),
    Integer(i64),
    Integer(u32),
    Integer(i32),
    Integer(u16),
    Integer(i16),
    Integer(u8),
    Integer(i8),

    Bytes(Vec<u8>),
    Bytes(&[u8]),

    Float(Float),
    Float(f64),
    Float(f32),

    Text(String),
    Text(&str),

    Bool(bool),

    Array(&[Value]),
    Array(Vec<Value>),

    Map(&[(Value, Value)]),
    Map(Vec<(Value, Value)>),
}

impl From<u128> for Value {
    #[inline]
    fn from(value: u128) -> Self {
        if let Ok(x) = Integer::try_from(value) {
            return Value::Integer(x);
        }

        let mut bytes = &value.to_be_bytes()[..];
        while !bytes.is_empty() && bytes[0] == 0 {
            bytes = &bytes[1..];
        }

        Value::Tag(ciborium_ll::tag::BIGPOS, Value::Bytes(bytes.into()).into())
    }
}

impl From<i128> for Value {
    #[inline]
    fn from(value: i128) -> Self {
        if let Ok(x) = Integer::try_from(value) {
            return Value::Integer(x);
        }

        let (tag, raw) = match value.is_negative() {
            true => (ciborium_ll::tag::BIGNEG, value as u128 ^ !0),
            false => (ciborium_ll::tag::BIGPOS, value as u128),
        };

        let mut bytes = &raw.to_be_bytes()[..];
        while !bytes.is_empty() && bytes[0] == 0 {
            bytes = &bytes[1..];
        }

        Value::Tag(tag, Value::Bytes(bytes.into()).into())
    }
}

impl From<char> for Value {
    #[inline]
    fn from(value: char) -> Self {
        let mut v = String::with_capacity(1);
        v.push(value);
        Value::Text(v)
    }
}
