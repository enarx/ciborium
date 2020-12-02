// SPDX-License-Identifier: Apache-2.0

//! Serde serialization support for CBOR

mod error;

use crate::basic::*;
use crate::io::Write;
use crate::value::Float;
pub use error::Error;

use alloc::string::ToString;
use core::convert::TryFrom;

use serde::{ser, Serialize as _};

struct Encoder<T: Write>(T);

impl<T: Write> Encoder<T> {
    #[inline]
    fn save(&mut self, title: impl Into<Title>) -> Result<(), Error<T::Error>> {
        let title = title.into();

        let major: u8 = match title.0 {
            Major::Positive => 0,
            Major::Negative => 1,
            Major::Bytes => 2,
            Major::Text => 3,
            Major::Array => 4,
            Major::Map => 5,
            Major::Tag => 6,
            Major::Other => 7,
        };

        let minor: u8 = match title.1 {
            Minor::Immediate(x) => x.into(),
            Minor::Subsequent1(..) => 24,
            Minor::Subsequent2(..) => 25,
            Minor::Subsequent4(..) => 26,
            Minor::Subsequent8(..) => 27,
            Minor::Indeterminate => 31,
        };

        self.0.write_all(&[(major << 5) | minor])?;
        self.0.write_all(title.1.as_ref())?;
        Ok(())
    }

    #[inline]
    fn bignum(&mut self, negative: bool, v: u128) -> Result<(), Error<T::Error>> {
        if let Ok(v) = u64::try_from(v) {
            return self.save(Title(
                match negative {
                    false => Major::Positive,
                    true => Major::Negative,
                },
                Minor::from(v),
            ));
        }

        let bytes = v.to_be_bytes();
        let length = bytes.iter().skip_while(|x| **x == 0).count();

        self.save(match negative {
            false => Title::TAG_BIGPOS,
            true => Title::TAG_BIGNEG,
        })?;

        self.save(Title(Major::Bytes, length.into()))?;
        Ok(self.0.write_all(&bytes[bytes.len() - length..])?)
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
    T::Error: core::fmt::Debug,
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
        self.save(if v { Title::TRUE } else { Title::FALSE })
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<(), Self::Error> {
        self.serialize_i64(v.into())
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<(), Self::Error> {
        self.serialize_i64(v.into())
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<(), Self::Error> {
        self.serialize_i64(v.into())
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<(), Self::Error> {
        match v.is_negative() {
            false => self.save(Title(Major::Positive, Minor::from(v as u64))),
            true => self.save(Title(Major::Negative, Minor::from(v as u64 ^ !0))),
        }
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<(), Self::Error> {
        match v.is_negative() {
            false => self.bignum(false, v as u128),
            true => self.bignum(true, v as u128 ^ !0),
        }
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<(), Self::Error> {
        self.serialize_u64(v.into())
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<(), Self::Error> {
        self.serialize_u64(v.into())
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<(), Self::Error> {
        self.serialize_u64(v.into())
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<(), Self::Error> {
        self.save(Title(Major::Positive, Minor::from(v)))
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<(), Self::Error> {
        self.bignum(false, v)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<(), Self::Error> {
        self.serialize_f64(v.into())
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<(), Self::Error> {
        let v = Float::from(v);

        let minor = if let Ok(x) = half::f16::try_from(v) {
            Minor::Subsequent2(x.to_be_bytes())
        } else if let Ok(x) = f32::try_from(v) {
            Minor::Subsequent4(x.to_be_bytes())
        } else {
            Minor::Subsequent8(f64::from(v).to_be_bytes())
        };

        self.save(Title(Major::Other, minor))
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<(), Self::Error> {
        self.serialize_str(&v.to_string())
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<(), Self::Error> {
        let bytes = v.as_bytes();
        self.save(Title(Major::Text, bytes.len().into()))?;
        Ok(self.0.write_all(bytes)?)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Self::Error> {
        self.save(Title(Major::Bytes, v.len().into()))?;
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
        self.serialize_str(variant)
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
        self.save(Title(Major::Map, 1usize.into()))?;
        self.serialize_str(variant)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, length: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.save(Title(Major::Array, length.into()))?;
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
        self.save(Title(Major::Map, 1usize.into()))?;
        self.serialize_str(variant)?;
        self.save(Title(Major::Array, length.into()))?;
        Ok(CollectionEncoder {
            encoder: self,
            ending: false,
        })
    }

    #[inline]
    fn serialize_map(self, length: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.save(Title(Major::Map, length.into()))?;
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
        self.save(Title(Major::Map, length.into()))?;
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
        self.save(Title(Major::Map, 1usize.into()))?;
        self.serialize_str(variant)?;
        self.save(Title(Major::Map, length.into()))?;
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
    T::Error: core::fmt::Debug,
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
    T::Error: core::fmt::Debug,
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
    T::Error: core::fmt::Debug,
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
    T::Error: core::fmt::Debug,
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
    T::Error: core::fmt::Debug,
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
    T::Error: core::fmt::Debug,
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
    T::Error: core::fmt::Debug,
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
    W::Error: core::fmt::Debug,
{
    let mut encoder = Encoder::from(writer);
    value.serialize(&mut encoder)?;
    Ok(encoder.0.flush()?)
}
