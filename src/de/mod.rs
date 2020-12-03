// SPDX-License-Identifier: Apache-2.0

//! Serde deserialization support for CBOR

mod error;

use crate::basic::*;
use crate::io::Read;
use crate::value::Float;
pub use error::Error;

use alloc::{string::String, vec::Vec};
use core::convert::{TryFrom, TryInto};

use serde::de::{self, Deserialize as _, Deserializer as _};

#[inline]
fn length<T: core::fmt::Debug>(title: Title, offset: usize) -> Result<Option<usize>, Error<T>> {
    Ok(title
        .1
        .try_into()
        .map_err(|_| Error::semantic(offset, "unsuppored length"))?)
}

#[derive(Debug)]
struct Io<T: Read> {
    reader: T,
    buffer: Option<(Title, usize)>,
    offset: usize,
}

impl<T: Read> Read for Io<T> {
    type Error = T::Error;

    #[inline]
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        self.reader.read_exact(data)?;
        self.offset += data.len();
        Ok(())
    }
}

impl<T: Read> From<T> for Io<T> {
    fn from(value: T) -> Self {
        Self {
            reader: value,
            buffer: None,
            offset: 0,
        }
    }
}

impl<T: Read> Io<T> {
    #[inline]
    fn push(&mut self, item: (Title, usize)) {
        self.buffer = Some(item);
    }

    #[inline]
    fn pull(&mut self, skip_tag: bool) -> Result<(Title, usize), Error<T::Error>> {
        if let Some((title, offset)) = self.buffer.take() {
            if title.0 != Major::Tag || !skip_tag {
                return Ok((title, offset));
            }
        }

        loop {
            let (title, offset) = match self.buffer.take() {
                Some(x) => x,
                None => {
                    let offset = self.offset;

                    let mut prefix = [0u8];
                    self.read_exact(&mut prefix)?;

                    let mut title = Title::try_from(prefix[0]).or(Err(Error::Syntax(offset)))?;
                    self.read_exact(title.1.as_mut())?;

                    (title, offset)
                }
            };

            if title.0 != Major::Tag || !skip_tag {
                return Ok((title, offset));
            }
        }
    }
}

struct Deserializer<T>(T);

impl<'a, T: Read> Deserializer<&'a mut Io<T>>
where
    T::Error: core::fmt::Debug,
{
    #[inline]
    fn chunked(
        &mut self,
        maj: Major,
        mut fnc: impl FnMut(&mut Io<T>, usize, usize) -> Result<(), Error<T::Error>>,
    ) -> Result<(), Error<T::Error>> {
        let mut chunked = 0usize;

        loop {
            let (title, offset) = self.0.pull(true)?;

            if chunked > 0 && title == Title::BREAK {
                chunked -= 1;
                if chunked == 0 {
                    return Ok(());
                }
            }

            if title.0 != maj {
                return Err(Error::Syntax(offset));
            }

            if let Some(len) = length(title, offset)? {
                fnc(self.0, len, offset)?;
                if chunked == 0 {
                    return Ok(());
                }
            } else {
                chunked += 1;
            }
        }
    }

    #[inline]
    fn bignum(&mut self, msg: &str) -> Result<u128, Error<T::Error>> {
        let mut buffer = 0u128.to_ne_bytes();
        let mut index = 0;

        self.chunked(Major::Bytes, |rdr, len, offset| {
            Ok(for _ in 0..len {
                let mut byte = [0];
                rdr.read_exact(&mut byte)?;

                // Skip leading zeroes.
                if index == 0 && byte[0] == 0 {
                    continue;
                }

                if index >= buffer.len() {
                    return Err(Error::semantic(offset, msg));
                }

                buffer[index] = byte[0];
                index += 1;
            })
        })?;

        buffer[0..index].reverse();
        Ok(u128::from_le_bytes(buffer))
    }

    #[inline]
    fn float<N: TryFrom<Float>>(&mut self, msg: &str) -> Result<N, Error<T::Error>> {
        let (title, offset) = self.0.pull(true)?;

        let float = match (title.0, title.1) {
            (Major::Other, Minor::Subsequent2(x)) => half::f16::from_be_bytes(x).into(),
            (Major::Other, Minor::Subsequent4(x)) => f32::from_be_bytes(x).into(),
            (Major::Other, Minor::Subsequent8(x)) => f64::from_be_bytes(x).into(),
            _ => return Err(Error::semantic(offset, msg)),
        };

        N::try_from(float).map_err(|_| Error::semantic(offset, msg))
    }

    #[inline]
    fn integer<N: TryFrom<u128> + TryFrom<i128>>(
        &mut self,
        msg: &str,
    ) -> Result<N, Error<T::Error>> {
        let (title, offset) = self.0.pull(false)?;

        let signed = match title {
            Title::TAG_BIGPOS => {
                return Ok(self
                    .bignum(msg)?
                    .try_into()
                    .map_err(|_| Error::semantic(offset + 1, msg))?)
            }

            Title::TAG_BIGNEG => {
                let raw = self.bignum(msg)?;

                if raw.leading_zeros() == 0 {
                    return Err(Error::semantic(offset + 1, msg));
                }

                raw as i128 ^ !0
            }

            Title(Major::Positive, minor) => Option::<u64>::from(minor)
                .ok_or(Error::Syntax(offset))?
                .into(),

            Title(Major::Negative, minor) => Option::<u64>::from(minor)
                .map(|x| x as i128 ^ !0)
                .ok_or(Error::Syntax(offset))?,

            _ => return Err(Error::semantic(offset, msg)),
        };

        Ok(signed
            .try_into()
            .map_err(|_| Error::semantic(offset, msg))?)
    }
}

impl<'a, 'de, T: Read> de::Deserializer<'de> for Deserializer<&'a mut Io<T>>
where
    T::Error: core::fmt::Debug,
{
    type Error = Error<T::Error>;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Self::Error> {
        loop {
            let item = self.0.pull(false)?;
            self.0.push(item);

            return match item.0 {
                Title::TAG_BIGPOS => self.deserialize_u128(v),
                Title::TAG_BIGNEG => self.deserialize_i128(v),

                Title(Major::Positive, _) => self.deserialize_u128(v),
                Title(Major::Negative, _) => self.deserialize_i128(v),
                Title(Major::Bytes, _) => self.deserialize_byte_buf(v),
                Title(Major::Text, _) => self.deserialize_string(v),
                Title(Major::Array, _) => self.deserialize_seq(v),
                Title(Major::Map, _) => self.deserialize_map(v),

                Title(Major::Other, Minor::Subsequent2(_)) => self.deserialize_f64(v),
                Title(Major::Other, Minor::Subsequent4(_)) => self.deserialize_f64(v),
                Title(Major::Other, Minor::Subsequent8(_)) => self.deserialize_f64(v),
                Title::FALSE => self.deserialize_bool(v),
                Title::TRUE => self.deserialize_bool(v),
                Title::UNDEFINED => self.deserialize_option(v),
                Title::NULL => self.deserialize_option(v),

                Title(Major::Tag, _) => {
                    self.0.pull(false)?;
                    continue;
                }

                _ => Err(Error::semantic(item.1, "unknown type")),
            };
        }
    }

    #[inline]
    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0.pull(true)? {
            (Title::FALSE, _) => visitor.visit_bool(false),
            (Title::TRUE, _) => visitor.visit_bool(true),
            (_, offset) => Err(Error::semantic(offset, "expected boolean")),
        }
    }

    #[inline]
    fn deserialize_i8<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i8(self.integer("expected i8")?)
    }

    #[inline]
    fn deserialize_i16<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i16(self.integer("expected i16")?)
    }

    #[inline]
    fn deserialize_i32<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i32(self.integer("expected i32")?)
    }

    #[inline]
    fn deserialize_i64<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i64(self.integer("expected i64")?)
    }

    #[inline]
    fn deserialize_i128<V: de::Visitor<'de>>(
        mut self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_i128(self.integer("expected i128")?)
    }

    #[inline]
    fn deserialize_u8<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u8(self.integer("expected u8")?)
    }

    #[inline]
    fn deserialize_u16<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u16(self.integer("expected u16")?)
    }

    #[inline]
    fn deserialize_u32<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u32(self.integer("expected u32")?)
    }

    #[inline]
    fn deserialize_u64<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u64(self.integer("expected u64")?)
    }

    #[inline]
    fn deserialize_u128<V: de::Visitor<'de>>(
        mut self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_u128(self.integer("expected u128")?)
    }

    #[inline]
    #[allow(clippy::float_cmp)]
    fn deserialize_f32<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f32(self.float("expected f32")?)
    }

    #[inline]
    fn deserialize_f64<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f64(self.float("expected f64")?)
    }

    #[inline]
    fn deserialize_char<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let offset = self.0.offset;
        let string = String::deserialize(self)?;
        let mut chars = string.chars();
        if let Some(c) = chars.next() {
            if chars.next().is_none() {
                return visitor.visit_char(c);
            }
        }

        Err(Error::semantic(offset, "expected char"))
    }

    #[inline]
    fn deserialize_str<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_string(visitor)
    }

    #[inline]
    fn deserialize_string<V: de::Visitor<'de>>(
        mut self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let mut string = String::new();

        self.chunked(Major::Text, |rdr, len, offset| {
            let mut buf = vec![0; len];
            rdr.read_exact(&mut buf[..])?;

            match core::str::from_utf8(&buf) {
                Ok(s) => Ok(string.push_str(s)),
                Err(..) => Err(Error::Syntax(offset)),
            }
        })?;

        visitor.visit_string(string)
    }

    #[inline]
    fn deserialize_bytes<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_byte_buf(visitor)
    }

    #[inline]
    fn deserialize_byte_buf<V: de::Visitor<'de>>(
        mut self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let mut buffer = Vec::new();

        self.chunked(Major::Bytes, |rdr, len, _| {
            let cur = buffer.len();
            buffer.resize(cur + len, 0);
            Ok(rdr.read_exact(&mut buffer[cur..])?)
        })?;

        visitor.visit_byte_buf(buffer)
    }

    #[inline]
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0.pull(true)? {
            (Title::UNDEFINED, _) | (Title::NULL, _) => visitor.visit_none(),
            item => {
                self.0.push(item);
                visitor.visit_some(self)
            }
        }
    }

    #[inline]
    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.0.pull(true)? {
            (Title::UNDEFINED, _) => visitor.visit_unit(),
            (Title::NULL, _) => visitor.visit_unit(),
            (_, offset) => Err(Error::semantic(offset, "expected unit")),
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

    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(Deserializer::new(self.0, Major::Array, "expected array")?)
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
    fn deserialize_map<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_map(Deserializer::new(self.0, Major::Map, "expected map")?)
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
        let (title, offset) = self.0.pull(true)?;

        match title {
            Title(Major::Map, Minor::Immediate(1)) => visitor.visit_enum(self),
            Title(Major::Text, ..) => {
                self.0.push((title, offset));
                visitor.visit_enum(self)
            }
            _ => Err(Error::semantic(offset, "expected enum")),
        }
    }

    #[inline]
    fn deserialize_identifier<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_ignored_any<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_any(visitor)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, T: Read> Deserializer<(&'a mut Io<T>, Option<usize>, usize)>
where
    T::Error: core::fmt::Debug,
{
    fn new(io: &'a mut Io<T>, major: Major, msg: &str) -> Result<Self, Error<T::Error>> {
        let (title, offset) = io.pull(true)?;

        if title.0 != major {
            return Err(Error::semantic(offset, msg));
        }

        Ok(Self((io, length(title, offset)?, 0)))
    }
}

impl<'a, 'de, T: Read> de::SeqAccess<'de> for Deserializer<(&'a mut Io<T>, Option<usize>, usize)>
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
            Some(len) if self.0 .2 >= len => return Ok(None),
            Some(_) => self.0 .2 += 1,
            None => match self.0 .0.pull(true)? {
                (Title::BREAK, _) => return Ok(None),
                item => self.0 .0.push(item),
            },
        }

        seed.deserialize(Deserializer(&mut *self.0 .0)).map(Some)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.0 .1
    }
}

impl<'a, 'de, T: Read> de::MapAccess<'de> for Deserializer<(&'a mut Io<T>, Option<usize>, usize)>
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
            Some(len) if self.0 .2 >= len => return Ok(None),
            Some(_) => self.0 .2 += 1,
            None => match self.0 .0.pull(true)? {
                (Title::BREAK, _) => return Ok(None),
                item => self.0 .0.push(item),
            },
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

impl<'a, 'de, T: Read> de::EnumAccess<'de> for Deserializer<&'a mut Io<T>>
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

impl<'a, 'de, T: Read> de::VariantAccess<'de> for Deserializer<&'a mut Io<T>>
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
