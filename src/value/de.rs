// SPDX-License-Identifier: Apache-2.0

use super::{Error, Integer, Value};

use alloc::{string::String, vec::Vec};
use core::convert::TryFrom;
use core::iter::Peekable;

use serde::de::{self, Deserializer as _};
use serde::forward_to_deserialize_any;

impl<'a> From<Integer> for de::Unexpected<'a> {
    #[inline]
    fn from(value: Integer) -> Self {
        u64::try_from(value)
            .map(|x| de::Unexpected::Unsigned(x))
            .unwrap_or_else(|_| {
                i64::try_from(value)
                    .map(|x| de::Unexpected::Signed(x))
                    .unwrap_or_else(|_| de::Unexpected::Other("large integer"))
            })
    }
}

impl<'a> From<&'a Value> for de::Unexpected<'a> {
    #[inline]
    fn from(value: &'a Value) -> Self {
        match value {
            Value::Bool(x) => Self::Bool(*x),
            Value::Integer(x) => Self::from(*x),
            Value::Float(x) => Self::Float(f64::from(*x)),
            Value::Bytes(x) => Self::Bytes(x),
            Value::Text(x) => Self::Str(x),
            Value::Array(..) => Self::Seq,
            Value::Map(..) => Self::Map,
            Value::Null => Self::Other("null"),
        }
    }
}

macro_rules! mkvisit {
    ($($f:ident($v:ty)),+ $(,)?) => {
        $(
            #[inline]
            fn $f<E: de::Error>(self, v: $v) -> Result<Self::Value, E> {
                Ok(v.into())
            }
        )+
    };
}

struct Visitor;

impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "a valid CBOR item")
    }

    mkvisit! {
        visit_bool(bool),
        visit_f32(f32),
        visit_f64(f64),

        visit_i8(i8),
        visit_i16(i16),
        visit_i32(i32),
        visit_i64(i64),
        visit_i128(i128),

        visit_u8(u8),
        visit_u16(u16),
        visit_u32(u32),
        visit_u64(u64),
        visit_u128(u128),

        visit_char(char),
        visit_str(&str),
        visit_borrowed_str(&'de str),
        visit_string(String),

        visit_bytes(&[u8]),
        visit_borrowed_bytes(&'de [u8]),
        visit_byte_buf(Vec<u8>),
    }

    #[inline]
    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    #[inline]
    fn visit_some<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(self)
    }

    #[inline]
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    #[inline]
    fn visit_newtype_struct<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(self)
    }

    #[inline]
    fn visit_seq<A: de::SeqAccess<'de>>(self, mut acc: A) -> Result<Self::Value, A::Error> {
        let mut seq = Vec::with_capacity(acc.size_hint().unwrap_or(0));

        while let Some(elem) = acc.next_element()? {
            seq.push(elem);
        }

        Ok(Value::Array(seq))
    }

    #[inline]
    fn visit_map<A: de::MapAccess<'de>>(self, mut acc: A) -> Result<Self::Value, A::Error> {
        let mut map = Vec::<(Value, Value)>::with_capacity(acc.size_hint().unwrap_or(0));

        while let Some(kv) = acc.next_entry()? {
            map.push(kv);
        }

        Ok(Value::Map(map))
    }
}

impl<'de> de::Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(Visitor)
    }
}

struct Deserializer<T>(T);

impl<'a, 'de> de::Deserializer<'de> for Deserializer<&'a Value> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0 {
            Value::Bytes(x) => visitor.visit_bytes(x),
            Value::Text(x) => visitor.visit_str(x),
            Value::Array(x) => visitor.visit_seq(Deserializer(x.iter())),
            Value::Map(x) => visitor.visit_map(Deserializer(x.iter().peekable())),
            Value::Bool(x) => visitor.visit_bool(*x),
            Value::Null => visitor.visit_none(),

            Value::Integer(x) => {
                if let Ok(x) = u8::try_from(*x) {
                    visitor.visit_u8(x)
                } else if let Ok(x) = i8::try_from(*x) {
                    visitor.visit_i8(x)
                } else if let Ok(x) = u16::try_from(*x) {
                    visitor.visit_u16(x)
                } else if let Ok(x) = i16::try_from(*x) {
                    visitor.visit_i16(x)
                } else if let Ok(x) = u32::try_from(*x) {
                    visitor.visit_u32(x)
                } else if let Ok(x) = i32::try_from(*x) {
                    visitor.visit_i32(x)
                } else if let Ok(x) = u64::try_from(*x) {
                    visitor.visit_u64(x)
                } else if let Ok(x) = i64::try_from(*x) {
                    visitor.visit_i64(x)
                } else if let Ok(x) = u128::try_from(*x) {
                    visitor.visit_u128(x)
                } else if let Ok(x) = i128::try_from(*x) {
                    visitor.visit_i128(x)
                } else {
                    unreachable!()
                }
            }

            Value::Float(x) => {
                if let Ok(x) = f32::try_from(*x) {
                    visitor.visit_f32(x)
                } else if let Ok(x) = f64::try_from(*x) {
                    visitor.visit_f64(x)
                } else {
                    unreachable!()
                }
            }
        }
    }

    forward_to_deserialize_any! {
            i8 i16 i32 i64 i128
            u8 u16 u32 u64 u128
            bool f32 f64 char str string bytes byte_buf seq map

    //        option unit unit_struct newtype_struct tuple
    //        tuple_struct struct enum

            identifier ignored_any
        }

    #[inline]
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0 {
            Value::Null => visitor.visit_none(),
            x => visitor.visit_some(Self(x)),
        }
    }

    #[inline]
    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0 {
            Value::Null => visitor.visit_unit(),
            _ => Err(de::Error::invalid_type(self.0.into(), &"null")),
        }
    }

    #[inline]
    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_tuple<V: de::Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.0 {
            Value::Map(x) if x.len() == 1 => visitor.visit_enum(Deserializer(x.iter())),
            _ => Err(de::Error::invalid_type(self.0.into(), &"map")),
        }
    }
}

impl<'a, 'de, T: Iterator<Item = &'a Value>> de::SeqAccess<'de> for Deserializer<T> {
    type Error = Error;

    #[inline]
    fn next_element_seed<U: de::DeserializeSeed<'de>>(
        &mut self,
        seed: U,
    ) -> Result<Option<U::Value>, Self::Error> {
        match self.0.next() {
            None => Ok(None),
            Some(v) => seed.deserialize(Deserializer(v)).map(Some),
        }
    }
}

impl<'a, 'de, T: Iterator<Item = &'a (Value, Value)>> de::MapAccess<'de>
    for Deserializer<Peekable<T>>
{
    type Error = Error;

    #[inline]
    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.0.peek() {
            None => Ok(None),
            Some(x) => Ok(Some(seed.deserialize(Deserializer(&x.0))?)),
        }
    }

    #[inline]
    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        seed.deserialize(Deserializer(&self.0.next().unwrap().1))
    }
}

impl<'a, 'de, T: Iterator<Item = &'a (Value, Value)>> de::EnumAccess<'de> for Deserializer<T> {
    type Error = Error;
    type Variant = Deserializer<&'a Value>;

    #[inline]
    fn variant_seed<V: de::DeserializeSeed<'de>>(
        mut self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        match self.0.next() {
            Some((k, v)) => Ok((seed.deserialize(Deserializer(k))?, Deserializer(v))),
            None => Err(de::Error::invalid_length(0, &"exatly one")),
        }
    }
}

impl<'a, 'de> de::VariantAccess<'de> for Deserializer<&'a Value> {
    type Error = Error;

    #[inline]
    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.0 {
            Value::Null => Ok(()),
            _ => Err(de::Error::invalid_type(self.0.into(), &"unit")),
        }
    }

    #[inline]
    fn newtype_variant_seed<U: de::DeserializeSeed<'de>>(
        self,
        seed: U,
    ) -> Result<U::Value, Self::Error> {
        seed.deserialize(Deserializer(self.0))
    }

    #[inline]
    fn tuple_variant<V: de::Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Deserializer(self.0).deserialize_seq(visitor)
    }

    #[inline]
    fn struct_variant<V: de::Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Deserializer(self.0).deserialize_map(visitor)
    }
}

impl Value {
    /// Deserializes the `Value` into an object
    #[inline]
    pub fn deserialized<'de, T: de::Deserialize<'de>>(&self) -> Result<T, Error> {
        T::deserialize(Deserializer(self))
    }
}
