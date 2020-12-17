// SPDX-License-Identifier: Apache-2.0

//! Serde deserialization support for CBOR

mod error;

use crate::basic::*;
use crate::io::Read;
pub use error::Error;

use alloc::{string::String, vec::Vec};

use serde::de::{self, Deserializer as _};
use serde::forward_to_deserialize_any;

struct Deserializer<'b, R: Read> {
    decoder: Decoder<R>,
    scratch: &'b mut [u8],
    recurse: usize,
}

impl<'de, 'a, 'b, R: Read> Deserializer<'b, R>
where
    R::Error: core::fmt::Debug,
{
    #[inline]
    fn recurse<V, F: FnOnce(&mut Self) -> Result<V, Error<R::Error>>>(
        &mut self,
        func: F,
    ) -> Result<V, Error<R::Error>> {
        if self.recurse == 0 {
            return Err(Error::RecursionLimitExceeded);
        }

        self.recurse -= 1;
        let result = func(self);
        self.recurse += 1;
        result
    }
}

impl<'de, 'a, 'b, R: Read> de::Deserializer<'de> for &'a mut Deserializer<'b, R>
where
    R::Error: core::fmt::Debug,
{
    type Error = Error<R::Error>;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Self::Error> {
        loop {
            let offset = self.decoder.offset();
            return match self.decoder.pull()? {
                Header::Positive(x) => v.visit_u64(x),
                Header::Negative(x) => match x.leading_zeros() {
                    0 => v.visit_i128(x as i128 ^ !0),
                    _ => v.visit_i64(x as i64 ^ !0),
                },

                Header::Bytes(len) => match len {
                    Some(len) if len <= self.scratch.len() => {
                        self.decoder.read_exact(&mut self.scratch[..len])?;
                        v.visit_bytes(&self.scratch[..len])
                    }

                    len => {
                        let mut buffer = Vec::new();

                        let mut segments = self.decoder.bytes(len, &mut self.scratch[..]);
                        while let Some(mut segment) = segments.next()? {
                            while let Some(chunk) = segment.next()? {
                                buffer.extend_from_slice(chunk);
                            }
                        }

                        v.visit_byte_buf(buffer)
                    }
                },

                Header::Text(len) => match len {
                    Some(len) if len <= self.scratch.len() => {
                        self.decoder.read_exact(&mut self.scratch[..len])?;
                        match core::str::from_utf8(&self.scratch[..len]) {
                            Ok(s) => v.visit_str(s),
                            Err(..) => Err(Error::Syntax(offset)),
                        }
                    }

                    len => {
                        let mut buffer = String::new();

                        let mut segments = self.decoder.text(len, &mut self.scratch[..]);
                        while let Some(mut segment) = segments.next()? {
                            while let Some(chunk) = segment.next()? {
                                buffer.push_str(chunk);
                            }
                        }

                        v.visit_string(buffer)
                    }
                },

                Header::Array(len) => self.recurse(|me| v.visit_seq(Access(me, len))),
                Header::Map(len) => self.recurse(|me| v.visit_map(Access(me, len))),

                Header::Tag(TAG_BIGPOS) => {
                    let offset = self.decoder.offset();
                    match self.decoder.bigint() {
                        Err(None) => Err(Error::semantic(offset, "bigint too large")),
                        Err(Some(e)) => Err(e.into()),
                        Ok(raw) => v.visit_u128(raw),
                    }
                }

                Header::Tag(TAG_BIGNEG) => {
                    let offset = self.decoder.offset();
                    match self.decoder.bigint() {
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
        struct tuple
        identifier ignored_any
    }

    #[inline]
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            return match self.decoder.pull()? {
                Header::Simple(SIMPLE_UNDEFINED) => visitor.visit_none(),
                Header::Simple(SIMPLE_NULL) => visitor.visit_none(),
                Header::Tag(..) => continue,
                header => {
                    self.decoder.push(header);
                    visitor.visit_some(self)
                }
            };
        }
    }

    #[inline]
    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            let offset = self.decoder.offset();

            return match self.decoder.pull()? {
                Header::Simple(SIMPLE_UNDEFINED) => visitor.visit_unit(),
                Header::Simple(SIMPLE_NULL) => visitor.visit_unit(),
                Header::Tag(..) => continue,
                _ => Err(Error::semantic(offset, "expected unit")),
            };
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
        loop {
            let offset = self.decoder.offset();

            match self.decoder.pull()? {
                Header::Tag(..) => continue,
                Header::Map(Some(1)) => (),
                header @ Header::Text(..) => self.decoder.push(header),
                _ => return Err(Error::semantic(offset, "expected enum")),
            }

            return self.recurse(|me| visitor.visit_enum(Access(me, Some(0))));
        }
    }

    #[inline]
    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match (name, len) {
            ("@@TAG@@", 2) => visitor.visit_seq(TagAccess(self, 0)),
            _ => self.deserialize_any(visitor),
        }
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct Access<'a, 'b, R: Read>(&'a mut Deserializer<'b, R>, Option<usize>);

impl<'de, 'a, 'b, R: Read> de::SeqAccess<'de> for Access<'a, 'b, R>
where
    R::Error: core::fmt::Debug,
{
    type Error = Error<R::Error>;

    #[inline]
    fn next_element_seed<U: de::DeserializeSeed<'de>>(
        &mut self,
        seed: U,
    ) -> Result<Option<U::Value>, Self::Error> {
        match self.1 {
            Some(0) => return Ok(None),
            Some(x) => self.1 = Some(x - 1),
            None => match self.0.decoder.pull()? {
                Header::Break => return Ok(None),
                header => self.0.decoder.push(header),
            },
        }

        seed.deserialize(&mut *self.0).map(Some)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.1
    }
}

impl<'de, 'a, 'b, R: Read> de::MapAccess<'de> for Access<'a, 'b, R>
where
    R::Error: core::fmt::Debug,
{
    type Error = Error<R::Error>;

    #[inline]
    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.1 {
            Some(0) => return Ok(None),
            Some(x) => self.1 = Some(x - 1),
            None => match self.0.decoder.pull()? {
                Header::Break => return Ok(None),
                header => self.0.decoder.push(header),
            },
        }

        seed.deserialize(&mut *self.0).map(Some)
    }

    #[inline]
    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        seed.deserialize(&mut *self.0)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.1
    }
}

impl<'de, 'a, 'b, R: Read> de::EnumAccess<'de> for Access<'a, 'b, R>
where
    R::Error: core::fmt::Debug,
{
    type Error = Error<R::Error>;
    type Variant = Self;

    #[inline]
    fn variant_seed<V: de::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let variant = seed.deserialize(&mut *self.0)?;
        Ok((variant, self))
    }
}

impl<'de, 'a, 'b, R: Read> de::VariantAccess<'de> for Access<'a, 'b, R>
where
    R::Error: core::fmt::Debug,
{
    type Error = Error<R::Error>;

    #[inline]
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn newtype_variant_seed<U: de::DeserializeSeed<'de>>(
        self,
        seed: U,
    ) -> Result<U::Value, Self::Error> {
        seed.deserialize(&mut *self.0)
    }

    #[inline]
    fn tuple_variant<V: de::Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0.deserialize_any(visitor)
    }

    #[inline]
    fn struct_variant<V: de::Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0.deserialize_any(visitor)
    }
}

struct TagAccess<'a, 'b, R: Read>(&'a mut Deserializer<'b, R>, usize);

impl<'de, 'a, 'b, R: Read> de::Deserializer<'de> for &mut TagAccess<'a, 'b, R>
where
    R::Error: core::fmt::Debug,
{
    type Error = Error<R::Error>;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let offset = self.0.decoder.offset();

        match self.0.decoder.pull()? {
            Header::Tag(x) => visitor.visit_u64(x),
            _ => Err(Error::semantic(offset, "expected tag")),
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
        option unit unit_struct newtype_struct enum
    }
}

impl<'de, 'a, 'b, R: Read> de::SeqAccess<'de> for TagAccess<'a, 'b, R>
where
    R::Error: core::fmt::Debug,
{
    type Error = Error<R::Error>;

    #[inline]
    fn next_element_seed<U: de::DeserializeSeed<'de>>(
        &mut self,
        seed: U,
    ) -> Result<Option<U::Value>, Self::Error> {
        self.1 += 1;

        match self.1 {
            1 => seed.deserialize(self).map(Some),
            2 => seed.deserialize(&mut *self.0).map(Some),
            _ => Ok(None),
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(match self.1 {
            0 => 2,
            1 => 1,
            _ => 0,
        })
    }
}

/// Deserializes as CBOR from a type with [`impl ciborium::serde::de::Read`](crate::serde::de::Read)
#[inline]
pub fn from_reader<'de, T: de::Deserialize<'de>, R: Read>(reader: R) -> Result<T, Error<R::Error>>
where
    R::Error: core::fmt::Debug,
{
    let mut scratch = [0; 4096];

    let mut reader = Deserializer {
        decoder: reader.into(),
        scratch: &mut scratch,
        recurse: 256,
    };

    T::deserialize(&mut reader)
}
