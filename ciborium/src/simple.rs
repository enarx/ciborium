//! Contains helper types for dealing with CBOR simple values

use ciborium_ll::{simple, Decoder, Header};
use serde::{de, forward_to_deserialize_any, ser};

pub(crate) struct SimpleDeserializer<'d, R, D> {
    parent: core::marker::PhantomData<D>,
    decoder: &'d mut Decoder<R>,
}

impl<'d, R, D> SimpleDeserializer<'d, R, D> {
    pub fn new(decoder: &'d mut Decoder<R>) -> Self {
        Self {
            parent: core::marker::PhantomData,
            decoder,
        }
    }
}

impl<'de, 'd, D: de::Deserializer<'de>, R: ciborium_io::Read> de::Deserializer<'de>
    for &mut SimpleDeserializer<'d, R, D>
where
    R::Error: core::fmt::Debug,
{
    type Error = crate::de::Error<R::Error>;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let offset = self.decoder.offset();

        let Header::Simple(simple) = self.decoder.pull()? else {
                return Err(crate::de::Error::semantic(offset, "expected simple"));
            };
        visitor.visit_u8(simple)
    }

    #[inline]
    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        loop {
            let offset = self.decoder.offset();

            return match self.decoder.pull()? {
                Header::Tag(..) => continue,
                Header::Simple(simple::FALSE) => visitor.visit_bool(false),
                Header::Simple(simple::TRUE) => visitor.visit_bool(true),
                _ => Err(crate::de::Error::semantic(offset, "expected bool")),
            };
        }
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128
        u8 u16 u32 u64 u128
        f32 f64
        char str string
        bytes byte_buf
        seq map
        struct tuple tuple_struct
        identifier ignored_any
        option unit unit_struct newtype_struct enum
    }
}

#[derive(Debug)]
pub(crate) struct Error;

impl core::fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ser::StdError for Error {}

impl ser::Error for Error {
    fn custom<U: core::fmt::Display>(_msg: U) -> Self {
        Error
    }
}

pub(crate) struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = u8;
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn serialize_bool(self, _: bool) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_i8(self, _: i8) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_i16(self, _: i16) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_i32(self, _: i32) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_i64(self, _: i64) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_i128(self, _: i128) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<u8, Self::Error> {
        Ok(v)
    }

    #[inline]
    fn serialize_u16(self, _: u16) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_u32(self, _: u32) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_u64(self, _: u64) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_u128(self, _: u128) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_f32(self, _: f32) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_f64(self, _: f64) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_char(self, _: char) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_str(self, _: &str) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_bytes(self, _: &[u8]) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_none(self) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_some<U: ?Sized + ser::Serialize>(self, _: &U) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_unit(self) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
    ) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_newtype_struct<U: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _value: &U,
    ) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_newtype_variant<U: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &U,
    ) -> Result<u8, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_seq(self, _length: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_tuple(self, _length: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_map(self, _length: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl ser::SerializeSeq for Serializer {
    type Ok = u8;
    type Error = Error;

    #[inline]
    fn serialize_element<U: ?Sized + ser::Serialize>(&mut self, _value: &U) -> Result<(), Error> {
        Err(Error)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(Error)
    }
}

impl ser::SerializeTuple for Serializer {
    type Ok = u8;
    type Error = Error;

    #[inline]
    fn serialize_element<U: ?Sized + ser::Serialize>(&mut self, _value: &U) -> Result<(), Error> {
        Err(Error)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(Error)
    }
}

impl ser::SerializeTupleStruct for Serializer {
    type Ok = u8;
    type Error = Error;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(&mut self, _value: &U) -> Result<(), Error> {
        Err(Error)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(Error)
    }
}

impl ser::SerializeTupleVariant for Serializer {
    type Ok = u8;
    type Error = Error;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(&mut self, _value: &U) -> Result<(), Error> {
        Err(Error)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(Error)
    }
}

impl ser::SerializeMap for Serializer {
    type Ok = u8;
    type Error = Error;

    #[inline]
    fn serialize_key<U: ?Sized + ser::Serialize>(&mut self, _key: &U) -> Result<(), Error> {
        Err(Error)
    }

    #[inline]
    fn serialize_value<U: ?Sized + ser::Serialize>(&mut self, _value: &U) -> Result<(), Error> {
        Err(Error)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(Error)
    }
}

impl ser::SerializeStruct for Serializer {
    type Ok = u8;
    type Error = Error;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        _key: &'static str,
        _value: &U,
    ) -> Result<(), Error> {
        Err(Error)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(Error)
    }
}

impl ser::SerializeStructVariant for Serializer {
    type Ok = u8;
    type Error = Error;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        _key: &'static str,
        _value: &U,
    ) -> Result<(), Self::Error> {
        Err(Error)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(Error)
    }
}
