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

impl Value {
    /// Returns true if the `Value` is an `Integer`. Returns false otherwise.
    ///
    /// ```
    /// # use ciborium::value::Value;
    /// #
    /// let value = Value::Integer(17.into());
    ///
    /// assert!(value.is_integer());
    /// ```
    pub fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }

    /// If the `Value` is a `Integer`, returns the associated Integer data. Returns None
    /// otherwise.
    ///
    /// ```
    /// # use ciborium::value::Value;
    /// #
    /// let value = Value::Integer(17.into());
    ///
    /// // We can read the number
    /// assert_eq!(value.as_integer().unwrap().value(), &17_i128);
    /// ```
    pub fn as_integer(&self) -> Option<&Integer> {
        match *self {
            Value::Integer(ref int) => Some(int),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a `Bytes`. Returns false otherwise.
    ///
    /// ```
    /// # use ciborium::value::Value;
    /// #
    /// let value = Value::Bytes(vec![104, 101, 108, 108, 111]);
    ///
    /// assert!(value.is_bytes());
    /// ```
    pub fn is_bytes(&self) -> bool {
        self.as_bytes().is_some()
    }

    /// If the `Value` is a `Bytes`, returns the associated bytes vector. Returns None
    /// otherwise.
    ///
    /// ```
    /// # use ciborium::value::Value;
    /// #
    /// let value = Value::Bytes(vec![104, 101, 108, 108, 111]);
    ///
    /// assert_eq!(std::str::from_utf8(value.as_bytes().unwrap()).unwrap(), "hello");
    /// ```
    pub fn as_bytes(&self) -> Option<&Vec<u8>> {
        match *self {
            Value::Bytes(ref bytes) => Some(bytes),
            _ => None,
        }
    }

    /// If the `Value` is a `Bytes`, returns a mutable reference to the associated bytes vector. Returns None
    /// otherwise.
    ///
    /// ```
    /// # use ciborium::value::Value;
    /// #
    /// let mut value = Value::Bytes(vec![104, 101, 108, 108, 111]);
    /// value.as_bytes_mut().unwrap().clear();
    ///
    /// assert_eq!(value, Value::Bytes(vec![]));
    /// ```
    pub fn as_bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
        match *self {
            Value::Bytes(ref mut bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Returns true if the `Value` is an Array. Returns false otherwise.
    ///
    /// For any Value on which `is_array` returns true, `as_array` and
    /// `as_array_mut` are guaranteed to return the vector representing the
    /// array.
    ///
    /// ```
    /// # use ciborium::value::Value;
    /// #
    /// let value = Value::Array(
    ///     vec![
    ///         Value::Text(String::from("foo")),
    ///         Value::Text(String::from("bar"))
    ///     ]
    /// );
    ///
    /// assert!(value.is_array());
    /// ```
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// If the `Value` is an Array, returns the associated vector. Returns None
    /// otherwise.
    ///
    /// ```
    /// # use ciborium::value::Value;
    /// #
    /// let value = Value::Array(
    ///     vec![
    ///         Value::Text(String::from("foo")),
    ///         Value::Text(String::from("bar"))
    ///     ]
    /// );
    ///
    /// // The length of `value` is 2 elements.
    /// assert_eq!(value.as_array().unwrap().len(), 2);
    /// ```
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match *self {
            Value::Array(ref array) => Some(&*array),
            _ => None,
        }
    }

    /// If the `Value` is an Array, returns the associated mutable vector.
    /// Returns None otherwise.
    ///
    /// ```
    /// # use ciborium::value::Value;
    /// #
    /// let mut value = Value::Array(
    ///     vec![
    ///         Value::Text(String::from("foo")),
    ///         Value::Text(String::from("bar"))
    ///     ]
    /// );
    ///
    /// value.as_array_mut().unwrap().clear();
    /// assert_eq!(value, Value::Array(vec![]));
    /// ```
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match *self {
            Value::Array(ref mut list) => Some(list),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Map. Returns false otherwise.
    ///
    /// For any Value on which `is_map` returns true, `as_map` and
    /// `as_map_mut` are guaranteed to return the map representation of the
    /// map.
    ///
    /// ```
    /// # use ciborium::value::Value;
    /// #
    /// let value = Value::Map(
    ///     vec![
    ///         (Value::Text(String::from("foo")), Value::Text(String::from("bar")))
    ///     ]
    /// );
    ///
    /// assert!(value.is_map());
    /// ```
    pub fn is_map(&self) -> bool {
        self.as_map().is_some()
    }

    /// If the `Value` is a Map, returns the associated Map data. Returns None
    /// otherwise.
    ///
    /// ```
    /// # use ciborium::value::Value;
    /// #
    /// let value = Value::Map(
    ///     vec![
    ///         (Value::Text(String::from("foo")), Value::Text(String::from("bar")))
    ///     ]
    /// );
    ///
    /// // The length of data is 1 entry (1 key/value pair).
    /// assert_eq!(value.as_map().unwrap().len(), 1);
    ///
    /// // The content of the first element is what we expect
    /// assert_eq!(
    ///     value.as_map().unwrap().get(0).unwrap(),
    ///     &(Value::Text(String::from("foo")), Value::Text(String::from("bar")))
    /// );
    /// ```
    pub fn as_map(&self) -> Option<&Vec<(Value, Value)>> {
        match *self {
            Value::Map(ref map) => Some(map),
            _ => None,
        }
    }

    /// If the `Value` is a Map, returns the associated mutable Vec.
    /// Returns None otherwise.
    ///
    /// ```
    /// # use ciborium::value::Value;
    /// #
    /// let mut value = Value::Map(
    ///     vec![
    ///         (Value::Text(String::from("foo")), Value::Text(String::from("bar")))
    ///     ]
    /// );
    ///
    /// value.as_map_mut().unwrap().clear();
    /// assert_eq!(value, Value::Map(vec![]));
    /// ```
    pub fn as_map_mut(&mut self) -> Option<&mut Vec<(Value, Value)>> {
        match *self {
            Value::Map(ref mut map) => Some(map),
            _ => None,
        }
    }
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
