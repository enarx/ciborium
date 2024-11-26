//! Contains helper types for dealing with CBOR simple types

use serde::{de, de::Error as _, forward_to_deserialize_any, ser, Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "@@ST@@")]
enum Internal {
    /// The integer can either be 23, or (32..=255)
    #[serde(rename = "@@SIMPLETYPE@@")]
    SimpleType(u8),
}

/// A CBOR simple value
/// See https://datatracker.ietf.org/doc/html/rfc8949#section-3.3
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimpleType(pub u8);

impl<'de> Deserialize<'de> for SimpleType {
    #[inline]
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Internal::deserialize(deserializer)? {
            Internal::SimpleType(t) => Ok(SimpleType(t)),
        }
    }
}

impl Serialize for SimpleType {
    #[inline]
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Internal::SimpleType(self.0).serialize(serializer)
    }
}

pub(crate) struct SimpleTypeAccess<D> {
    parent: Option<D>,
    state: usize,
    typ: u8,
}

impl<D> SimpleTypeAccess<D> {
    pub fn new(parent: D, typ: u8) -> Self {
        Self {
            parent: Some(parent),
            state: 0,
            typ,
        }
    }
}

impl<'de, D: de::Deserializer<'de>> de::Deserializer<'de> for &mut SimpleTypeAccess<D> {
    type Error = D::Error;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.state += 1;
        match self.state {
            1 => visitor.visit_str("@@SIMPLETYPE@@"),
            _ => visitor.visit_u8(self.typ),
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

impl<'de, D: de::Deserializer<'de>> de::EnumAccess<'de> for SimpleTypeAccess<D> {
    type Error = D::Error;
    type Variant = Self;

    #[inline]
    fn variant_seed<V: de::DeserializeSeed<'de>>(
        mut self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let variant = seed.deserialize(&mut self)?;
        Ok((variant, self))
    }
}

impl<'de, D: de::Deserializer<'de>> de::VariantAccess<'de> for SimpleTypeAccess<D> {
    type Error = D::Error;

    #[inline]
    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(Self::Error::custom("expected simple type"))
    }

    #[inline]
    fn newtype_variant_seed<U: de::DeserializeSeed<'de>>(
        mut self,
        seed: U,
    ) -> Result<U::Value, Self::Error> {
        seed.deserialize(self.parent.take().unwrap())
    }

    #[inline]
    fn tuple_variant<V: de::Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(self)
    }

    #[inline]
    fn struct_variant<V: de::Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Err(Self::Error::custom("expected simple_type"))
    }
}

impl<'de, D: de::Deserializer<'de>> de::SeqAccess<'de> for SimpleTypeAccess<D> {
    type Error = D::Error;

    #[inline]
    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        if self.state < 2 {
            return Ok(Some(seed.deserialize(self)?));
        }

        Ok(match self.parent.take() {
            Some(x) => Some(seed.deserialize(x)?),
            None => None,
        })
    }
}
