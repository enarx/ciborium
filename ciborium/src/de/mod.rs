// SPDX-License-Identifier: Apache-2.0

//! Serde deserialization support for CBOR

mod error;

pub use error::Error;

use alloc::{string::String, vec::Vec};

use crate::tag::TagAccess;
use ciborium_io::{EndOfSliceError, Read, SliceReader};
use ciborium_ll::*;
use serde::de::{self, value::BytesDeserializer, Deserializer as _};

trait Expected<E: de::Error> {
    fn expected(self, kind: &'static str) -> E;
}

impl<E: de::Error> Expected<E> for Header {
    #[inline]
    fn expected(self, kind: &'static str) -> E {
        de::Error::invalid_type(
            match self {
                Header::Positive(x) => de::Unexpected::Unsigned(x),
                Header::Negative(x) => de::Unexpected::Signed(x as i64 ^ !0),
                Header::Bytes(..) => de::Unexpected::Other("bytes"),
                Header::Text(..) => de::Unexpected::Other("string"),

                Header::Array(..) => de::Unexpected::Seq,
                Header::Map(..) => de::Unexpected::Map,

                Header::Tag(..) => de::Unexpected::Other("tag"),

                Header::Simple(simple::FALSE) => de::Unexpected::Bool(false),
                Header::Simple(simple::TRUE) => de::Unexpected::Bool(true),
                Header::Simple(simple::NULL) => de::Unexpected::Other("null"),
                Header::Simple(simple::UNDEFINED) => de::Unexpected::Other("undefined"),
                Header::Simple(..) => de::Unexpected::Other("simple"),

                Header::Float(x) => de::Unexpected::Float(x),
                Header::Break => de::Unexpected::Other("break"),
            },
            &kind,
        )
    }
}

/// Deserializer
pub struct Deserializer<'slice, 'b, R> {
    decoder: Decoder<R>,
    scratch: &'b mut [u8],
    recurse: usize,
    borrow: BorrowSource<'slice, R>,
}

enum ByteSource<'slice> {
    Borrowed(&'slice [u8]),
    Scratch(usize),
}

type BorrowFn<'slice, R> = fn(&mut Decoder<R>, usize) -> Option<&'slice [u8]>;

#[derive(Copy, Clone)]
enum BorrowSource<'slice, R> {
    None,
    Slice(BorrowFn<'slice, R>),
}

impl<'slice, R> BorrowSource<'slice, R> {
    #[inline]
    fn try_borrow(&self, decoder: &mut Decoder<R>, len: usize) -> Option<&'slice [u8]> {
        match self {
            BorrowSource::None => None,
            BorrowSource::Slice(func) => func(decoder, len),
        }
    }

    #[inline]
    fn can_borrow(&self) -> bool {
        matches!(self, BorrowSource::Slice(_))
    }

    #[inline]
    fn none() -> Self {
        BorrowSource::None
    }

    #[inline]
    fn from_slice(func: BorrowFn<'slice, R>) -> Self {
        BorrowSource::Slice(func)
    }
}

fn borrow_from_slice<'slice>(
    decoder: &mut Decoder<&mut SliceReader<'slice>>,
    len: usize,
) -> Option<&'slice [u8]> {
    decoder.try_borrow_slice(len)
}

fn noop(_: u8) {}

impl<'slice, 'b, R: Read> Deserializer<'slice, 'b, R>
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

    #[inline]
    fn try_borrow_bytes(&mut self, len: usize) -> Result<Option<&'slice [u8]>, Error<R::Error>> {
        if let Some(slice) = self.borrow.try_borrow(&mut self.decoder, len) {
            return Ok(Some(slice));
        }
        Ok(None)
    }

    #[inline]
    fn read_bytes(&mut self, len: usize) -> Result<Option<ByteSource<'slice>>, Error<R::Error>> {
        if let Some(slice) = self.try_borrow_bytes(len)? {
            return Ok(Some(ByteSource::Borrowed(slice)));
        }

        if len <= self.scratch.len() {
            self.decoder.read_exact(&mut self.scratch[..len])?;
            return Ok(Some(ByteSource::Scratch(len)));
        }

        Ok(None)
    }

    #[inline]
    fn discard_bytes(&mut self, len: Option<usize>) -> Result<(), Error<R::Error>> {
        if let Some(len) = len {
            if self.try_borrow_bytes(len)?.is_some() {
                return Ok(());
            }
        }

        let mut segments = self.decoder.bytes(len);
        while let Some(mut segment) = segments.pull()? {
            while segment.pull(self.scratch)?.is_some() {}
        }
        Ok(())
    }

    #[inline]
    fn discard_text(&mut self, len: Option<usize>) -> Result<(), Error<R::Error>> {
        if let Some(len) = len {
            let offset = self.decoder.offset();

            if let Some(slice) = self.try_borrow_bytes(len)? {
                match core::str::from_utf8(slice) {
                    Ok(_) => return Ok(()),
                    Err(_) => return Err(Error::Syntax(offset)),
                }
            }
        }

        let mut segments = self.decoder.text(len);
        while let Some(mut segment) = segments.pull()? {
            while segment.pull(self.scratch)?.is_some() {}
        }
        Ok(())
    }

    #[inline]
    fn skip_value(&mut self) -> Result<(), Error<R::Error>> {
        let header = self.decoder.pull()?;
        self.skip_value_from_header(header)
    }

    #[inline]
    fn skip_value_from_header(&mut self, header: Header) -> Result<(), Error<R::Error>> {
        self.recurse(|me| me.skip_value_inner(header))
    }

    fn skip_value_inner(&mut self, header: Header) -> Result<(), Error<R::Error>> {
        match header {
            Header::Positive(..)
            | Header::Negative(..)
            | Header::Float(..)
            | Header::Simple(..) => Ok(()),

            Header::Bytes(len) => self.discard_bytes(len),
            Header::Text(len) => self.discard_text(len),

            Header::Array(len) => {
                match len {
                    Some(len) => {
                        for _ in 0..len {
                            self.skip_value()?;
                        }
                    }
                    None => loop {
                        match self.decoder.pull()? {
                            Header::Break => break,
                            header => self.skip_value_from_header(header)?,
                        }
                    },
                }
                Ok(())
            }

            Header::Map(len) => {
                match len {
                    Some(len) => {
                        for _ in 0..len {
                            self.skip_value()?;
                            self.skip_value()?;
                        }
                    }
                    None => loop {
                        match self.decoder.pull()? {
                            Header::Break => break,
                            header => {
                                self.skip_value_from_header(header)?;
                                self.skip_value()?;
                            }
                        }
                    },
                }
                Ok(())
            }

            Header::Tag(..) => self.skip_value(),

            Header::Break => Err(header.expected("non-break")),
        }
    }

    #[inline]
    fn integer<A: FnMut(u8)>(
        &mut self,
        mut header: Option<Header>,
        should_append: bool,
        mut append: A,
    ) -> Result<(bool, u128), Error<R::Error>> {
        loop {
            let header = match header.take() {
                Some(h) => h,
                None => self.decoder.pull()?,
            };

            let neg = match header {
                Header::Positive(x) => return Ok((false, x.into())),
                Header::Negative(x) => return Ok((true, x.into())),
                Header::Tag(tag::BIGPOS) => false,
                Header::Tag(tag::BIGNEG) => true,
                Header::Tag(..) => continue,
                header => return Err(header.expected("integer")),
            };

            let mut buffer = [0u8; 16];
            let mut value = [0u8; 16];
            let mut index = 0usize;

            return match self.decoder.pull()? {
                Header::Bytes(len) => {
                    let mut segments = self.decoder.bytes(len);
                    while let Some(mut segment) = segments.pull()? {
                        while let Some(chunk) = segment.pull(&mut buffer)? {
                            for b in chunk {
                                match index {
                                    16 => {
                                        if should_append {
                                            for v in value {
                                                append(v);
                                            }
                                            append(*b);
                                            index = 17; // Indicate overflow, see below
                                            continue;
                                        }
                                        return Err(de::Error::custom("bigint too large"));
                                    }
                                    17 => {
                                        debug_assert!(should_append);
                                        append(*b);
                                        continue;
                                    }
                                    0 if *b == 0 => continue, // Skip leading zeros
                                    _ => value[index] = *b,
                                }

                                index += 1;
                            }
                        }
                    }

                    if index == 17 {
                        Ok((false, 0))
                    } else {
                        value[..index].reverse();
                        Ok((neg, u128::from_le_bytes(value)))
                    }
                }

                h => Err(h.expected("bytes")),
            };
        }
    }
}

impl<'de, 'a, 'slice, 'b, R: Read> de::Deserializer<'de> for &'a mut Deserializer<'slice, 'b, R>
where
    'slice: 'de,
    R::Error: core::fmt::Debug,
{
    type Error = Error<R::Error>;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let header = self.decoder.pull()?;
        self.decoder.push(header);

        match header {
            Header::Positive(..) => self.deserialize_u64(visitor),
            Header::Negative(x) => match i64::try_from(x) {
                Ok(..) => self.deserialize_i64(visitor),
                Err(..) => self.deserialize_i128(visitor),
            },

            Header::Bytes(len) => match len {
                Some(len) if self.borrow.can_borrow() || len <= self.scratch.len() => {
                    self.deserialize_bytes(visitor)
                }
                _ => self.deserialize_byte_buf(visitor),
            },

            Header::Text(len) => match len {
                Some(len) if self.borrow.can_borrow() || len <= self.scratch.len() => {
                    self.deserialize_str(visitor)
                }
                _ => self.deserialize_string(visitor),
            },

            Header::Array(..) => self.deserialize_seq(visitor),
            Header::Map(..) => self.deserialize_map(visitor),

            Header::Tag(tag) => {
                let _: Header = self.decoder.pull()?;

                match tag {
                    tag::BIGPOS | tag::BIGNEG => {
                        let mut bytes = Vec::new();
                        let result =
                            match self.integer(Some(Header::Tag(tag)), true, |b| bytes.push(b))? {
                                (false, _) if !bytes.is_empty() => {
                                    let access =
                                        TagAccess::new(BytesDeserializer::new(&bytes), Some(tag));
                                    return visitor.visit_enum(access);
                                }
                                (false, raw) => return visitor.visit_u128(raw),
                                (true, raw) => i128::try_from(raw).map(|x| x ^ !0),
                            };

                        match result {
                            Ok(x) => visitor.visit_i128(x),
                            Err(..) => Err(de::Error::custom("integer too large")),
                        }
                    }

                    _ => self.recurse(|me| {
                        let access = TagAccess::new(me, Some(tag));
                        visitor.visit_enum(access)
                    }),
                }
            }

            Header::Float(..) => self.deserialize_f64(visitor),

            Header::Simple(simple::FALSE) => self.deserialize_bool(visitor),
            Header::Simple(simple::TRUE) => self.deserialize_bool(visitor),
            Header::Simple(simple::NULL) => self.deserialize_option(visitor),
            Header::Simple(simple::UNDEFINED) => self.deserialize_option(visitor),
            h @ Header::Simple(..) => Err(h.expected("known simple value")),

            h @ Header::Break => Err(h.expected("non-break")),
        }
    }

    #[inline]
    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            let offset = self.decoder.offset();

            return match self.decoder.pull()? {
                Header::Tag(..) => continue,
                Header::Simple(simple::FALSE) => visitor.visit_bool(false),
                Header::Simple(simple::TRUE) => visitor.visit_bool(true),
                _ => Err(Error::semantic(offset, "expected bool")),
            };
        }
    }

    #[inline]
    fn deserialize_f32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_f64(visitor)
    }

    #[inline]
    fn deserialize_f64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            return match self.decoder.pull()? {
                Header::Tag(..) => continue,
                Header::Float(x) => visitor.visit_f64(x),
                h => Err(h.expected("float")),
            };
        }
    }

    fn deserialize_i8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let result = match self.integer(None, false, noop)? {
            (false, raw) => i64::try_from(raw),
            (true, raw) => i64::try_from(raw).map(|x| x ^ !0),
        };

        match result {
            Ok(x) => visitor.visit_i64(x),
            Err(..) => Err(de::Error::custom("integer too large")),
        }
    }

    fn deserialize_i128<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let result = match self.integer(None, false, noop)? {
            (false, raw) => i128::try_from(raw),
            (true, raw) => i128::try_from(raw).map(|x| x ^ !0),
        };

        match result {
            Ok(x) => visitor.visit_i128(x),
            Err(..) => Err(de::Error::custom("integer too large")),
        }
    }

    fn deserialize_u8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let result = match self.integer(None, false, noop)? {
            (false, raw) => u64::try_from(raw),
            (true, ..) => return Err(de::Error::custom("unexpected negative integer")),
        };

        match result {
            Ok(x) => visitor.visit_u64(x),
            Err(..) => Err(de::Error::custom("integer too large")),
        }
    }

    fn deserialize_u128<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.integer(None, false, noop)? {
            (false, raw) => visitor.visit_u128(raw),
            (true, ..) => Err(de::Error::custom("unexpected negative integer")),
        }
    }

    fn deserialize_char<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            let offset = self.decoder.offset();
            let header = self.decoder.pull()?;

            return match header {
                Header::Tag(..) => continue,

                Header::Text(Some(len)) if len <= 4 => {
                    let mut buf = [0u8; 4];
                    self.decoder.read_exact(&mut buf[..len])?;

                    match core::str::from_utf8(&buf[..len]) {
                        Ok(s) => match s.chars().count() {
                            1 => visitor.visit_char(s.chars().next().unwrap()),
                            _ => Err(header.expected("char")),
                        },
                        Err(..) => Err(Error::Syntax(offset)),
                    }
                }

                _ => Err(header.expected("char")),
            };
        }
    }

    fn deserialize_str<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            let offset = self.decoder.offset();

            return match self.decoder.pull()? {
                Header::Tag(..) => continue,

                Header::Text(Some(len)) => match self.read_bytes(len)? {
                    Some(ByteSource::Borrowed(slice)) => match core::str::from_utf8(slice) {
                        Ok(s) => visitor.visit_borrowed_str(s),
                        Err(..) => Err(Error::Syntax(offset)),
                    },
                    Some(ByteSource::Scratch(len)) => {
                        match core::str::from_utf8(&self.scratch[..len]) {
                            Ok(s) => visitor.visit_str(s),
                            Err(..) => Err(Error::Syntax(offset)),
                        }
                    }
                    None => Err(Header::Text(Some(len)).expected("str")),
                },

                header => Err(header.expected("str")),
            };
        }
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            return match self.decoder.pull()? {
                Header::Tag(..) => continue,

                Header::Text(len) => {
                    let mut buffer = String::new();

                    let mut segments = self.decoder.text(len);
                    while let Some(mut segment) = segments.pull()? {
                        while let Some(chunk) = segment.pull(self.scratch)? {
                            buffer.push_str(chunk);
                        }
                    }

                    visitor.visit_string(buffer)
                }

                header => Err(header.expected("string")),
            };
        }
    }

    fn deserialize_bytes<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            return match self.decoder.pull()? {
                Header::Tag(..) => continue,

                Header::Bytes(Some(len)) => match self.read_bytes(len)? {
                    Some(ByteSource::Borrowed(slice)) => visitor.visit_borrowed_bytes(slice),
                    Some(ByteSource::Scratch(len)) => visitor.visit_bytes(&self.scratch[..len]),
                    None => Err(Header::Bytes(Some(len)).expected("bytes")),
                },

                Header::Array(len) => self.recurse(|me| {
                    let access = Access(me, len);
                    visitor.visit_seq(access)
                }),

                header => Err(header.expected("bytes")),
            };
        }
    }

    fn deserialize_byte_buf<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        loop {
            return match self.decoder.pull()? {
                Header::Tag(..) => continue,

                Header::Bytes(len) => {
                    let mut buffer = Vec::new();

                    let mut segments = self.decoder.bytes(len);
                    while let Some(mut segment) = segments.pull()? {
                        while let Some(chunk) = segment.pull(self.scratch)? {
                            buffer.extend_from_slice(chunk);
                        }
                    }

                    visitor.visit_byte_buf(buffer)
                }

                Header::Array(len) => self.recurse(|me| {
                    let access = Access(me, len);
                    visitor.visit_seq(access)
                }),

                header => Err(header.expected("byte buffer")),
            };
        }
    }

    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            return match self.decoder.pull()? {
                Header::Tag(..) => continue,

                Header::Array(len) => self.recurse(|me| {
                    let access = Access(me, len);
                    visitor.visit_seq(access)
                }),

                Header::Bytes(len) => {
                    let mut buffer = Vec::new();

                    let mut segments = self.decoder.bytes(len);
                    while let Some(mut segment) = segments.pull()? {
                        while let Some(chunk) = segment.pull(self.scratch)? {
                            buffer.extend_from_slice(chunk);
                        }
                    }

                    visitor.visit_seq(BytesAccess::<R>(0, buffer, core::marker::PhantomData))
                }

                header => Err(header.expected("array")),
            };
        }
    }

    fn deserialize_map<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            return match self.decoder.pull()? {
                Header::Tag(..) => continue,

                Header::Map(len) => self.recurse(|me| {
                    let access = Access(me, len);
                    visitor.visit_map(access)
                }),

                header => Err(header.expected("map")),
            };
        }
    }

    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_tuple<V: de::Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_identifier<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        loop {
            let offset = self.decoder.offset();

            return match self.decoder.pull()? {
                Header::Tag(..) => continue,

                Header::Text(Some(len)) => match self.read_bytes(len)? {
                    Some(ByteSource::Borrowed(slice)) => match core::str::from_utf8(slice) {
                        Ok(s) => visitor.visit_borrowed_str(s),
                        Err(..) => Err(Error::Syntax(offset)),
                    },
                    Some(ByteSource::Scratch(len)) => {
                        match core::str::from_utf8(&self.scratch[..len]) {
                            Ok(s) => visitor.visit_str(s),
                            Err(..) => Err(Error::Syntax(offset)),
                        }
                    }
                    None => Err(Header::Text(Some(len)).expected("str or bytes")),
                },
                Header::Bytes(Some(len)) => match self.read_bytes(len)? {
                    Some(ByteSource::Borrowed(slice)) => visitor.visit_borrowed_bytes(slice),
                    Some(ByteSource::Scratch(len)) => visitor.visit_bytes(&self.scratch[..len]),
                    None => Err(Header::Bytes(Some(len)).expected("str or bytes")),
                },

                header => Err(header.expected("str or bytes")),
            };
        }
    }

    fn deserialize_ignored_any<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.skip_value()?;
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.decoder.pull()? {
            Header::Simple(simple::UNDEFINED) => visitor.visit_none(),
            Header::Simple(simple::NULL) => visitor.visit_none(),
            header => {
                self.decoder.push(header);
                visitor.visit_some(self)
            }
        }
    }

    #[inline]
    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            return match self.decoder.pull()? {
                Header::Simple(simple::UNDEFINED) => visitor.visit_unit(),
                Header::Simple(simple::NULL) => visitor.visit_unit(),
                Header::Tag(..) => continue,
                header => Err(header.expected("unit")),
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
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if name == "@@TAG@@" {
            let tag = match self.decoder.pull()? {
                Header::Tag(x) => Some(x),
                header => {
                    self.decoder.push(header);
                    None
                }
            };

            return self.recurse(|me| {
                let access = TagAccess::new(me, tag);
                visitor.visit_enum(access)
            });
        }

        loop {
            match self.decoder.pull()? {
                Header::Tag(..) => continue,
                Header::Map(Some(1)) => (),
                header @ Header::Text(..) => self.decoder.push(header),
                header => return Err(header.expected("enum")),
            }

            return self.recurse(|me| {
                let access = Access(me, Some(0));
                visitor.visit_enum(access)
            });
        }
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct Access<'slice, 'a, 'b, R>(&'a mut Deserializer<'slice, 'b, R>, Option<usize>);

impl<'de, 'slice, 'a, 'b, R: Read> de::SeqAccess<'de> for Access<'slice, 'a, 'b, R>
where
    'slice: 'de,
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

impl<'de, 'slice, 'a, 'b, R: Read> de::MapAccess<'de> for Access<'slice, 'a, 'b, R>
where
    'slice: 'de,
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

impl<'de, 'slice, 'a, 'b, R: Read> de::EnumAccess<'de> for Access<'slice, 'a, 'b, R>
where
    'slice: 'de,
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

impl<'de, 'slice, 'a, 'b, R: Read> de::VariantAccess<'de> for Access<'slice, 'a, 'b, R>
where
    'slice: 'de,
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

struct BytesAccess<R>(usize, Vec<u8>, core::marker::PhantomData<R>);

impl<'de, R: Read> de::SeqAccess<'de> for BytesAccess<R>
where
    R::Error: core::fmt::Debug,
{
    type Error = Error<R::Error>;

    #[inline]
    fn next_element_seed<U: de::DeserializeSeed<'de>>(
        &mut self,
        seed: U,
    ) -> Result<Option<U::Value>, Self::Error> {
        use de::IntoDeserializer;

        if self.0 < self.1.len() {
            let byte = self.1[self.0];
            self.0 += 1;
            seed.deserialize(byte.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.1.len() - self.0)
    }
}

/// Deserializes as CBOR from a byte slice using a 4KB buffer on the stack.
///
/// Unlike [`from_reader`](from_reader), this function accepts types that implement
/// `Deserialize<'de>` rather than `DeserializeOwned`, allowing deserialized data
/// to have lifetime parameters tied to the input slice.
///
/// # Example
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct Data {
///     name: String,
///     age: u32,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // CBOR encoding of {"name": "Alice", "age": 30}
/// let input = b"\xa2\x64name\x65Alice\x63age\x18\x1e";
/// let data: Data = ciborium::de::from_slice(input)?;
/// assert_eq!(data, Data { name: "Alice".to_string(), age: 30 });
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn from_slice<'de, T>(slice: &'de [u8]) -> Result<T, Error<EndOfSliceError>>
where
    T: de::Deserialize<'de>,
{
    let mut scratch = [0; 4096];
    from_slice_with_buffer(slice, &mut scratch)
}

/// Deserializes as CBOR from a byte slice, using a caller-specific buffer as a
/// temporary scratch space.
///
/// This is the slice equivalent of [`from_reader_with_buffer`](from_reader_with_buffer),
/// supporting zero-copy deserialization for types that implement `Deserialize<'de>`.
#[inline]
pub fn from_slice_with_buffer<'de, T>(
    slice: &'de [u8],
    scratch_buffer: &mut [u8],
) -> Result<T, Error<EndOfSliceError>>
where
    T: de::Deserialize<'de>,
{
    let mut reader = SliceReader::new(slice);
    let borrow = BorrowSource::from_slice(borrow_from_slice);
    let mut deserializer = Deserializer {
        decoder: (&mut reader).into(),
        scratch: scratch_buffer,
        recurse: 256,
        borrow,
    };

    T::deserialize(&mut deserializer)
}

/// Deserializes as CBOR from a byte slice with a specified maximum recursion limit.
///
/// This is the slice equivalent of [`from_reader_with_recursion_limit`](from_reader_with_recursion_limit),
/// supporting zero-copy deserialization. Inputs nested beyond the specified limit
/// will result in [`Error::RecursionLimitExceeded`].
///
/// Set a high recursion limit at your own risk (of stack exhaustion)!
#[inline]
pub fn from_slice_with_recursion_limit<'de, T>(
    slice: &'de [u8],
    recurse_limit: usize,
) -> Result<T, Error<EndOfSliceError>>
where
    T: de::Deserialize<'de>,
{
    let mut scratch = [0; 4096];

    let mut reader = SliceReader::new(slice);
    let borrow = BorrowSource::from_slice(borrow_from_slice);
    let mut deserializer = Deserializer {
        decoder: (&mut reader).into(),
        scratch: &mut scratch,
        recurse: recurse_limit,
        borrow,
    };

    T::deserialize(&mut deserializer)
}

/// Deserializes as CBOR from a type with [`impl
/// ciborium_io::Read`](ciborium_io::Read) using a 4KB buffer on the stack.
///
/// If you want to deserialize faster at the cost of more memory, consider using
/// [`from_reader_with_buffer`](from_reader_with_buffer) with a larger buffer,
/// for example 64KB.
#[inline]
pub fn from_reader<T: de::DeserializeOwned, R: Read>(reader: R) -> Result<T, Error<R::Error>>
where
    R::Error: core::fmt::Debug,
{
    let mut scratch = [0; 4096];
    from_reader_with_buffer(reader, &mut scratch)
}

/// Deserializes as CBOR from a type with [`impl
/// ciborium_io::Read`](ciborium_io::Read), using a caller-specific buffer as a
/// temporary scratch space.
#[inline]
pub fn from_reader_with_buffer<T: de::DeserializeOwned, R: Read>(
    reader: R,
    scratch_buffer: &mut [u8],
) -> Result<T, Error<R::Error>>
where
    R::Error: core::fmt::Debug,
{
    let mut reader = Deserializer {
        decoder: reader.into(),
        scratch: scratch_buffer,
        recurse: 256,
        borrow: BorrowSource::none(),
    };

    T::deserialize(&mut reader)
}

/// Deserializes as CBOR from a type with [`impl ciborium_io::Read`](ciborium_io::Read), with
/// a specified maximum recursion limit.  Inputs that are nested beyond the specified limit
/// will result in [`Error::RecursionLimitExceeded`] .
///
/// Set a high recursion limit at your own risk (of stack exhaustion)!
#[inline]
pub fn from_reader_with_recursion_limit<T: de::DeserializeOwned, R: Read>(
    reader: R,
    recurse_limit: usize,
) -> Result<T, Error<R::Error>>
where
    R::Error: core::fmt::Debug,
{
    let mut scratch = [0; 4096];

    let mut reader = Deserializer {
        decoder: reader.into(),
        scratch: &mut scratch,
        recurse: recurse_limit,
        borrow: BorrowSource::none(),
    };

    T::deserialize(&mut reader)
}

/// Returns a deserializer with a specified scratch buffer
#[inline]
pub fn deserializer_from_reader_with_buffer<R: Read>(
    reader: R,
    scratch_buffer: &mut [u8],
) -> Deserializer<'static, '_, R>
where
    R::Error: core::fmt::Debug,
{
    Deserializer {
        decoder: reader.into(),
        scratch: scratch_buffer,
        recurse: 256,
        borrow: BorrowSource::none(),
    }
}

/// Returns a deserializer with a specified scratch buffer
/// amd maximum recursion limit. Inputs that are nested beyond the specified limit
/// will result in [`Error::RecursionLimitExceeded`] .
///
/// Set a high recursion limit at your own risk (of stack exhaustion)!
#[inline]
pub fn deserializer_from_reader_with_buffer_and_recursion_limit<R: Read>(
    reader: R,
    scratch_buffer: &mut [u8],
    recurse_limit: usize,
) -> Deserializer<'static, '_, R>
where
    R::Error: core::fmt::Debug,
{
    Deserializer {
        decoder: reader.into(),
        scratch: scratch_buffer,
        recurse: recurse_limit,
        borrow: BorrowSource::none(),
    }
}
