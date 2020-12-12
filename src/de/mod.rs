// SPDX-License-Identifier: Apache-2.0

//! Serde deserialization support for CBOR

mod error;

use crate::basic::*;
use crate::io::Read;
pub use error::Error;

use alloc::{string::String, vec::Vec};

use serde::de::{self, Deserializer as _};
use serde::forward_to_deserialize_any;

struct Deserializer<T>(T);

impl<'a, 'de, T: Read> de::Deserializer<'de> for Deserializer<&'a mut Decoder<T>>
where
    T::Error: core::fmt::Debug,
{
    type Error = Error<T::Error>;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Self::Error> {
        let mut scratch = [0u8; 4096];

        loop {
            let offset = self.0.offset();

            return match self.0.pull(false)? {
                Header::Positive(x) => v.visit_u64(x),
                Header::Negative(x) => match x.leading_zeros() {
                    0 => v.visit_i128(x as i128 ^ !0),
                    _ => v.visit_i64(x as i64 ^ !0),
                },

                Header::Bytes(len) => match len {
                    Some(len) if len <= scratch.len() => {
                        self.0.read_exact(&mut scratch[..len])?;
                        v.visit_bytes(&scratch[..len])
                    }

                    len => {
                        let mut buffer = Vec::new();

                        let mut segments = self.0.bytes(len, &mut scratch[..]);
                        while let Some(mut segment) = segments.next()? {
                            while let Some(chunk) = segment.next()? {
                                buffer.extend_from_slice(chunk);
                            }
                        }

                        v.visit_byte_buf(buffer)
                    }
                },

                Header::Text(len) => match len {
                    Some(len) if len <= scratch.len() => {
                        self.0.read_exact(&mut scratch[..len])?;
                        match core::str::from_utf8(&scratch[..len]) {
                            Ok(s) => v.visit_str(s),
                            Err(..) => Err(Error::Syntax(offset)),
                        }
                    }

                    len => {
                        let mut buffer = String::new();

                        let mut segments = self.0.text(len, &mut scratch[..]);
                        while let Some(mut segment) = segments.next()? {
                            while let Some(chunk) = segment.next()? {
                                buffer.push_str(chunk);
                            }
                        }

                        v.visit_string(buffer)
                    }
                },

                Header::Array(len) => v.visit_seq(Deserializer((self.0, len))),
                Header::Map(len) => v.visit_map(Deserializer((self.0, len))),

                Header::Tag(TAG_BIGPOS) => {
                    let offset = self.0.offset();
                    match self.0.bigint() {
                        Err(None) => Err(Error::semantic(offset, "bigint too large")),
                        Err(Some(e)) => Err(e.into()),
                        Ok(raw) => v.visit_u128(raw),
                    }
                }

                Header::Tag(TAG_BIGNEG) => {
                    let offset = self.0.offset();
                    match self.0.bigint() {
                        Err(None) => Err(Error::semantic(offset, "bigint too large")),
                        Err(Some(e)) => Err(e.into()),
                        Ok(raw) => {
                            if raw.leading_zeros() == 0 {
                                return Err(Error::semantic(offset, "bigint too large"));
                            }

                            v.visit_i128(raw as i128 ^ !0)
                        }
                    }
                }

                Header::Tag(..) => continue,

                Header::Float(x) => v.visit_f64(x),
                Header::Simple(SIMPLE_FALSE) => v.visit_bool(false),
                Header::Simple(SIMPLE_TRUE) => v.visit_bool(true),
                Header::Simple(SIMPLE_NULL) => v.visit_none(),
                Header::Simple(SIMPLE_UNDEFINED) => v.visit_none(),

                Header::Simple(..) => Err(Error::semantic(offset, "unknown simple value")),
                Header::Break => Err(Error::semantic(offset, "unexpected break")),
            };
        }
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128
        u8 u16 u32 u64 u128
        bool f32 f64
        char str string
        bytes byte_buf
        seq map
        struct tuple tuple_struct
        identifier ignored_any
    }

    #[inline]
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0.peek(true)? {
            Header::Simple(SIMPLE_UNDEFINED) => self.0.dump(),
            Header::Simple(SIMPLE_NULL) => self.0.dump(),
            _ => return visitor.visit_some(self),
        }

        visitor.visit_none()
    }

    #[inline]
    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let offset = self.0.offset();
        match self.0.pull(true)? {
            Header::Simple(SIMPLE_UNDEFINED) => visitor.visit_unit(),
            Header::Simple(SIMPLE_NULL) => visitor.visit_unit(),
            _ => Err(Error::semantic(offset, "expected unit")),
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
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let offset = self.0.offset();
        match self.0.peek(true)? {
            Header::Map(Some(1)) => self.0.dump(),
            Header::Text(..) => return visitor.visit_enum(self),
            _ => return Err(Error::semantic(offset, "expected enum")),
        }

        visitor.visit_enum(self)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, 'de, T: Read> de::SeqAccess<'de> for Deserializer<(&'a mut Decoder<T>, Option<usize>)>
where
    T::Error: core::fmt::Debug,
{
    type Error = Error<T::Error>;

    #[inline]
    fn next_element_seed<U: de::DeserializeSeed<'de>>(
        &mut self,
        seed: U,
    ) -> Result<Option<U::Value>, Self::Error> {
        match self.0 .1 {
            Some(0) => return Ok(None),
            Some(x) => self.0 .1 = Some(x - 1),
            None => {
                if Header::Break == self.0 .0.peek(false)? {
                    self.0 .0.dump();
                    return Ok(None);
                }
            }
        }

        seed.deserialize(Deserializer(&mut *self.0 .0)).map(Some)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.0 .1
    }
}

impl<'a, 'de, T: Read> de::MapAccess<'de> for Deserializer<(&'a mut Decoder<T>, Option<usize>)>
where
    T::Error: core::fmt::Debug,
{
    type Error = Error<T::Error>;

    #[inline]
    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.0 .1 {
            Some(0) => return Ok(None),
            Some(x) => self.0 .1 = Some(x - 1),
            None => {
                if Header::Break == self.0 .0.peek(false)? {
                    self.0 .0.dump();
                    return Ok(None);
                }
            }
        }

        Ok(Some(seed.deserialize(Deserializer(&mut *self.0 .0))?))
    }

    #[inline]
    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        seed.deserialize(Deserializer(&mut *self.0 .0))
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.0 .1
    }
}

impl<'a, 'de, T: Read> de::EnumAccess<'de> for Deserializer<&'a mut Decoder<T>>
where
    T::Error: core::fmt::Debug,
{
    type Error = Error<T::Error>;
    type Variant = Self;

    #[inline]
    fn variant_seed<V: de::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let variant = seed.deserialize(Deserializer(&mut *self.0))?;
        Ok((variant, self))
    }
}

impl<'a, 'de, T: Read> de::VariantAccess<'de> for Deserializer<&'a mut Decoder<T>>
where
    T::Error: core::fmt::Debug,
{
    type Error = Error<T::Error>;

    #[inline]
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn newtype_variant_seed<U: de::DeserializeSeed<'de>>(
        self,
        seed: U,
    ) -> Result<U::Value, Self::Error> {
        seed.deserialize(self)
    }

    #[inline]
    fn tuple_variant<V: de::Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn struct_variant<V: de::Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_map(visitor)
    }
}

/// Deserializes as CBOR from a type with `impl ciborium::serde::de::Read`
#[inline]
pub fn from_reader<'de, T: de::Deserialize<'de>, R: Read>(reader: R) -> Result<T, Error<R::Error>>
where
    R::Error: core::fmt::Debug,
{
    let mut io = reader.into();
    T::deserialize(Deserializer(&mut io))
}
