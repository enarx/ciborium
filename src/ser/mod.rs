// SPDX-License-Identifier: Apache-2.0

//! Serde serialization support for CBOR

mod error;
mod write;

use crate::basic::*;
pub use error::Error;
use write::Write;

use alloc::string::ToString;
use core::convert::TryFrom;

use serde::{ser, Serialize as _};

struct Encoder<T: Write>(T);

impl<T: Write> Encoder<T> {
    #[inline]
    fn save(&mut self, title: impl Into<Title>) -> Result<(), Error<T::Error>> {
        let title = title.into();
        let prefix = Prefix::from(title);
        self.0.write_all(prefix.as_ref())?;
        Ok(self.0.write_all(title.1.as_ref())?)
    }
}

impl<T: Write> From<T> for Encoder<T> {
    #[inline]
    fn from(writer: T) -> Self {
        Encoder(writer)
    }
}

impl<'a, T: Write> ser::Serializer for &'a mut Encoder<T>
where
    T::Error: 'static + ser::StdError,
{
    type Ok = ();
    type Error = Error<T::Error>;

    type SerializeSeq = CollectionEncoder<'a, T>;
    type SerializeTuple = CollectionEncoder<'a, T>;
    type SerializeTupleStruct = CollectionEncoder<'a, T>;
    type SerializeTupleVariant = CollectionEncoder<'a, T>;
    type SerializeMap = CollectionEncoder<'a, T>;
    type SerializeStruct = CollectionEncoder<'a, T>;
    type SerializeStructVariant = CollectionEncoder<'a, T>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<(), Self::Error> {
        self.save(v)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<(), Self::Error> {
        self.save(v)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<(), Self::Error> {
        self.save(v)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<(), Self::Error> {
        self.save(v)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<(), Self::Error> {
        self.save(v)
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<(), Self::Error> {
        if let Ok(title) = Title::try_from(v) {
            return self.save(title);
        }

        let (tag, v) = match v.is_negative() {
            false => (Title::TAG_BIGPOS, v as u128),
            true => (Title::TAG_BIGNEG, v as u128 ^ !0),
        };

        let bytes = v.to_be_bytes();
        let length = bytes.iter().skip_while(|x| **x == 0).count();

        self.save(tag)?;
        self.save(Title::from_length(Major::Bytes, length))?;
        Ok(self.0.write_all(&bytes[bytes.len() - length..])?)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<(), Self::Error> {
        self.save(v)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<(), Self::Error> {
        self.save(v)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<(), Self::Error> {
        self.save(v)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<(), Self::Error> {
        self.save(v)
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<(), Self::Error> {
        let bytes = v.to_be_bytes();
        let length = bytes.iter().skip_while(|x| **x == 0).count();

        self.save(Title::TAG_BIGPOS)?;
        self.save(Title::from_length(Major::Bytes, length))?;
        Ok(self.0.write_all(&bytes[bytes.len() - length..])?)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<(), Self::Error> {
        self.save(v)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<(), Self::Error> {
        self.save(v)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<(), Self::Error> {
        self.serialize_str(&v.to_string())
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<(), Self::Error> {
        let bytes = v.as_bytes();
        self.save(Title::from_length(Major::Text, bytes.len()))?;
        Ok(self.0.write_all(bytes)?)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Self::Error> {
        self.save(Title::from_length(Major::Bytes, v.len()))?;
        Ok(self.0.write_all(v)?)
    }

    #[inline]
    fn serialize_none(self) -> Result<(), Self::Error> {
        self.save(Title::NULL)
    }

    #[inline]
    fn serialize_some<U: ?Sized + ser::Serialize>(self, value: &U) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.serialize_none()
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<(), Self::Error> {
        self.save(Title::from_length(Major::Map, 1usize))?;
        self.serialize_str(variant)?;
        self.serialize_unit()
    }

    #[inline]
    fn serialize_newtype_struct<U: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        value: &U,
    ) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<U: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &U,
    ) -> Result<(), Self::Error> {
        self.save(Title::from_length(Major::Map, 1usize))?;
        self.serialize_str(variant)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, length: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.save(Title::from_length(Major::Array, length))?;
        Ok(CollectionEncoder {
            encoder: self,
            ending: length.is_none(),
        })
    }

    #[inline]
    fn serialize_tuple(self, length: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(length))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        length: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(length))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        length: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.save(Title::from_length(Major::Map, 1usize))?;
        self.serialize_str(variant)?;
        self.save(Title::from_length(Major::Array, length))?;
        Ok(CollectionEncoder {
            encoder: self,
            ending: false,
        })
    }

    #[inline]
    fn serialize_map(self, length: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.save(Title::from_length(Major::Map, length))?;
        Ok(CollectionEncoder {
            encoder: self,
            ending: length.is_none(),
        })
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        length: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.save(Title::from_length(Major::Map, length))?;
        Ok(CollectionEncoder {
            encoder: self,
            ending: false,
        })
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        length: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.save(Title::from_length(Major::Map, 1usize))?;
        self.serialize_str(variant)?;
        self.save(Title::from_length(Major::Map, length))?;
        Ok(CollectionEncoder {
            encoder: self,
            ending: false,
        })
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

macro_rules! end {
    () => {
        #[inline]
        fn end(self) -> Result<(), Self::Error> {
            if self.ending {
                self.encoder.save(Title::BREAK)?;
            }

            Ok(())
        }
    };
}

struct CollectionEncoder<'a, T: Write> {
    encoder: &'a mut Encoder<T>,
    ending: bool,
}

impl<'a, T: Write> ser::SerializeSeq for CollectionEncoder<'a, T>
where
    T::Error: 'static + ser::StdError,
{
    type Ok = ();
    type Error = Error<T::Error>;

    #[inline]
    fn serialize_element<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut *self.encoder)
    }

    end!();
}

impl<'a, T: Write> ser::SerializeTuple for CollectionEncoder<'a, T>
where
    T::Error: 'static + ser::StdError,
{
    type Ok = ();
    type Error = Error<T::Error>;

    #[inline]
    fn serialize_element<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut *self.encoder)
    }

    end!();
}

impl<'a, T: Write> ser::SerializeTupleStruct for CollectionEncoder<'a, T>
where
    T::Error: 'static + ser::StdError,
{
    type Ok = ();
    type Error = Error<T::Error>;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut *self.encoder)
    }

    end!();
}

impl<'a, T: Write> ser::SerializeTupleVariant for CollectionEncoder<'a, T>
where
    T::Error: 'static + ser::StdError,
{
    type Ok = ();
    type Error = Error<T::Error>;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut *self.encoder)
    }

    end!();
}

impl<'a, T: Write> ser::SerializeMap for CollectionEncoder<'a, T>
where
    T::Error: 'static + ser::StdError,
{
    type Ok = ();
    type Error = Error<T::Error>;

    #[inline]
    fn serialize_key<U: ?Sized + ser::Serialize>(&mut self, key: &U) -> Result<(), Self::Error> {
        key.serialize(&mut *self.encoder)
    }

    #[inline]
    fn serialize_value<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut *self.encoder)
    }

    end!();
}

impl<'a, T: Write> ser::SerializeStruct for CollectionEncoder<'a, T>
where
    T::Error: 'static + ser::StdError,
{
    type Ok = ();
    type Error = Error<T::Error>;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        key: &'static str,
        value: &U,
    ) -> Result<(), Self::Error> {
        key.serialize(&mut *self.encoder)?;
        value.serialize(&mut *self.encoder)?;
        Ok(())
    }

    end!();
}

impl<'a, T: Write> ser::SerializeStructVariant for CollectionEncoder<'a, T>
where
    T::Error: 'static + ser::StdError,
{
    type Ok = ();
    type Error = Error<T::Error>;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        key: &'static str,
        value: &U,
    ) -> Result<(), Self::Error> {
        key.serialize(&mut *self.encoder)?;
        value.serialize(&mut *self.encoder)
    }

    end!();
}

/// Serializes as CBOR into a type with `impl ciborium::ser::Write`
#[inline]
pub fn into_writer<T: ?Sized + ser::Serialize, W: Write>(
    value: &T,
    writer: W,
) -> Result<(), Error<W::Error>>
where
    W::Error: 'static + ser::StdError,
{
    let mut encoder = Encoder::from(writer);
    value.serialize(&mut encoder)?;
    Ok(encoder.0.flush()?)
}
