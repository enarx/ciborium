// SPDX-License-Identifier: Apache-2.0

//! A dynamic CBOR value

mod bytes;
mod de;
mod error;
mod float;
mod integer;
mod ser;

use alloc::{string::String, vec::Vec};

pub use bytes::Bytes;
pub use error::Error;
pub use float::{Float, TryFromFloatError};
pub use integer::Integer;

/// A representation of a dynamic CBOR value that can handled dynamically
#[non_exhaustive]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    /// An integer
    Integer(Integer),

    /// Bytes
    Bytes(Bytes),

    /// A float
    Float(Float),

    /// A string
    Text(String),

    /// A boolean
    Bool(bool),

    /// Null
    Null,

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
    Integer(u128),
    Integer(i128),
    Integer(u64),
    Integer(i64),
    Integer(u32),
    Integer(i32),
    Integer(u16),
    Integer(i16),
    Integer(u8),
    Integer(i8),

    Bytes(Bytes),
    Bytes(Vec<u8>),
    Bytes(&[u8]),

    Float(Float),
    Float(f64),
    Float(f32),

    Text(String),
    Text(char),
    Text(&str),

    Bool(bool),

    Array(&[Value]),
    Array(Vec<Value>),

    Map(&[(Value, Value)]),
    Map(Vec<(Value, Value)>),
}
