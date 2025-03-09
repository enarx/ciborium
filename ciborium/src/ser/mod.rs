// SPDX-License-Identifier: Apache-2.0

//! Serde serialization support for CBOR

mod error;

pub use error::Error;

use alloc::string::ToString;
use ciborium_io::Write;
use ciborium_ll::*;
use core::marker::PhantomData;
use serde::ser;

use crate::canonical::{Canonicalization, NoCanonicalization};

/// A serializer for CBOR.
///
/// # Example
/// ```rust
/// use serde::Serialize;
/// use ciborium::Serializer;
///
/// #[derive(serde::Serialize)]
/// struct Example {
///     a: u64,
///     aa: u64,
///     b: u64,
/// }
///
/// let example = Example { a: 42, aa: 420, b: 4200 };
///
/// let mut buffer = Vec::with_capacity(1024);
///
/// #[cfg(feature = "std")] {
///     use ciborium::canonical::Rfc8949;
///     let mut serializer = Serializer::<_, Rfc8949>::new(&mut buffer);
///     example.serialize(&mut serializer).unwrap();
///     assert_eq!(hex::encode(&buffer), "a36161182a61621910686261611901a4");
/// }
///
/// #[cfg(not(feature = "std"))] {
///     use ciborium::canonical::NoCanonicalization;
///     let mut serializer = Serializer::<_, NoCanonicalization>::new(&mut buffer);  // uses no canonicalization
///     example.serialize(&mut serializer).unwrap();
///     assert_eq!(hex::encode(&buffer), "a36161182a6261611901a46162191068");
/// }
/// ```
pub struct Serializer<W, C: Canonicalization> {
    encoder: Encoder<W>,

    /// PhantomData to allow for type parameterization based on canonicalization scheme.
    canonicalization: PhantomData<C>,
}

impl<W: Write, C: Canonicalization> Serializer<W, C> {
    /// Create a new CBOR serializer.
    ///
    /// `canonicalization` can be used to change the [CanonicalizationScheme] used for sorting
    /// output map and struct keys to ensure deterministic outputs.
    #[inline]
    pub fn new(encoder: impl Into<Encoder<W>>) -> Self {
        Self {
            encoder: encoder.into(),
            canonicalization: PhantomData,
        }
    }
}

impl<W: Write, C: Canonicalization> From<W> for Serializer<W, C> {
    #[inline]
    fn from(writer: W) -> Self {
        Self {
            encoder: writer.into(),
            canonicalization: PhantomData,
        }
    }
}

impl<W: Write, C: Canonicalization> From<Encoder<W>> for Serializer<W, C> {
    #[inline]
    fn from(writer: Encoder<W>) -> Self {
        Self {
            encoder: writer,
            canonicalization: PhantomData,
        }
    }
}

impl<'a, W: Write, C: Canonicalization> ser::Serializer for &'a mut Serializer<W, C>
where
    W::Error: core::fmt::Debug,
{
    type Ok = ();
    type Error = Error<W::Error>;

    type SerializeSeq = CollectionSerializer<'a, W, C>;
    type SerializeTuple = CollectionSerializer<'a, W, C>;
    type SerializeTupleStruct = CollectionSerializer<'a, W, C>;
    type SerializeTupleVariant = CollectionSerializer<'a, W, C>;
    type SerializeMap = CollectionSerializer<'a, W, C>;
    type SerializeStruct = CollectionSerializer<'a, W, C>;
    type SerializeStructVariant = CollectionSerializer<'a, W, C>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<(), Self::Error> {
        Ok(self.encoder.push(match v {
            false => Header::Simple(simple::FALSE),
            true => Header::Simple(simple::TRUE),
        })?)
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
        Ok(self.encoder.push(match v.is_negative() {
            false => Header::Positive(v as u64),
            true => Header::Negative(v as u64 ^ !0),
        })?)
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<(), Self::Error> {
        let (tag, raw) = match v.is_negative() {
            false => (tag::BIGPOS, v as u128),
            true => (tag::BIGNEG, v as u128 ^ !0),
        };

        match (tag, u64::try_from(raw)) {
            (tag::BIGPOS, Ok(x)) => return Ok(self.encoder.push(Header::Positive(x))?),
            (tag::BIGNEG, Ok(x)) => return Ok(self.encoder.push(Header::Negative(x))?),
            _ => {}
        }

        let bytes = raw.to_be_bytes();

        // Skip leading zeros.
        let mut slice = &bytes[..];
        while !slice.is_empty() && slice[0] == 0 {
            slice = &slice[1..];
        }

        self.encoder.push(Header::Tag(tag))?;
        self.encoder.push(Header::Bytes(Some(slice.len())))?;
        Ok(self.encoder.write_all(slice)?)
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
        Ok(self.encoder.push(Header::Positive(v))?)
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<(), Self::Error> {
        if let Ok(x) = u64::try_from(v) {
            return self.serialize_u64(x);
        }

        let bytes = v.to_be_bytes();

        // Skip leading zeros.
        let mut slice = &bytes[..];
        while !slice.is_empty() && slice[0] == 0 {
            slice = &slice[1..];
        }

        self.encoder.push(Header::Tag(tag::BIGPOS))?;
        self.encoder.push(Header::Bytes(Some(slice.len())))?;
        Ok(self.encoder.write_all(slice)?)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<(), Self::Error> {
        self.serialize_f64(v.into())
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<(), Self::Error> {
        Ok(self.encoder.push(Header::Float(v))?)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<(), Self::Error> {
        self.serialize_str(&v.to_string())
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<(), Self::Error> {
        let bytes = v.as_bytes();
        self.encoder.push(Header::Text(bytes.len().into()))?;
        Ok(self.encoder.write_all(bytes)?)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Self::Error> {
        self.encoder.push(Header::Bytes(v.len().into()))?;
        Ok(self.encoder.write_all(v)?)
    }

    #[inline]
    fn serialize_none(self) -> Result<(), Self::Error> {
        Ok(self.encoder.push(Header::Simple(simple::NULL))?)
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
        name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &U,
    ) -> Result<(), Self::Error> {
        if name != "@@TAG@@" || variant != "@@UNTAGGED@@" {
            self.encoder.push(Header::Map(Some(1)))?;
            self.serialize_str(variant)?;
        }

        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, length: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        CollectionSerializer::new(self, CollectionType::Array, length)
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
        name: &'static str,
        _index: u32,
        variant: &'static str,
        length: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        match (name, variant) {
            ("@@TAG@@", "@@TAGGED@@") => {
                CollectionSerializer::new(self, CollectionType::Tag, Some(length))
            }

            _ => {
                self.encoder.push(Header::Map(Some(1)))?;
                self.serialize_str(variant)?;
                CollectionSerializer::new(self, CollectionType::Array, Some(length))
            }
        }
    }

    #[inline]
    fn serialize_map(self, length: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        CollectionSerializer::new(self, CollectionType::Map, length)
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        length: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        CollectionSerializer::new(self, CollectionType::Map, Some(length))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        length: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.encoder.push(Header::Map(Some(1)))?;
        self.serialize_str(variant)?;
        CollectionSerializer::new(self, CollectionType::Map, Some(length))
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

macro_rules! end {
    () => {
        #[allow(unused_mut)]
        #[inline]
        fn end(mut self) -> Result<(), Self::Error> {
            match C::SCHEME {
                None => {
                    if self.length.is_none() {
                        // Not canonical and no length => indefinite length break.
                        self.serializer.encoder.push(Header::Break)?;
                    }
                }

                #[cfg(not(feature = "std"))]
                Some(_) => {}

                #[cfg(feature = "std")]
                Some(_scheme) => {
                    // Canonical serialization holds back writing headers, as it doesn't allow
                    // indefinite length structs. This allows us to always compute the length.
                    self.push_header(Some(self.cache_values.len()))?;

                    for value in self.cache_values.iter() {
                        self.serializer.encoder.write_all(&value)?;
                    }
                }
            }

            Ok(())
        }
    };
}

macro_rules! end_map {
    () => {
        #[allow(unused_mut)]
        #[inline]
        fn end(mut self) -> Result<(), Self::Error> {
            match C::SCHEME {
                None => {
                    if self.length.is_none() {
                        // Not canonical and no length => indefinite length break.
                        self.serializer.encoder.push(Header::Break)?;
                    }
                }

                #[cfg(not(feature = "std"))]
                Some(_) => unreachable!(),

                #[cfg(feature = "std")]
                Some(scheme) => {
                    use crate::canonical::CanonicalizationScheme;

                    // Canonical serialization holds back writing headers, as it doesn't allow
                    // indefinite length structs. This allows us to always compute the length.
                    self.push_header(Some(self.cache_keys.len()))?;

                    // Sort our cached output and write it to the encoder.
                    match scheme {
                        CanonicalizationScheme::Rfc8949 => {
                            // keys get sorted in lexicographical byte order
                            let keys = self.cache_keys;
                            let values = self.cache_values;

                            debug_assert_eq!(
                                keys.len(), values.len(),
                                "ciborium error: canonicalization failed, different number of keys and values?");

                            let mut pairs: Vec<_> =
                                keys.iter().zip(values.iter()).collect();

                            pairs.sort();

                            for (key, value) in pairs.iter() {
                                self.serializer.encoder.write_all(&key)?;
                                self.serializer.encoder.write_all(&value)?;
                            }
                        }
                        CanonicalizationScheme::Rfc7049 => {
                            // keys get sorted in length-first byte order
                            let keys = self.cache_keys;
                            let values = self.cache_values;

                            debug_assert_eq!(
                                keys.len(), values.len(),
                                "ciborium error: canonicalization failed, different number of keys and values?");

                            let mut pairs: Vec<_> =
                                keys.iter()
                                    .map(|key| (key.len(), key))  // length-first ordering
                                    .zip(values.iter())
                                    .collect();

                            pairs.sort();

                            for ((_, key), value) in pairs.iter() {
                                self.serializer.encoder.write_all(&key)?;
                                self.serializer.encoder.write_all(&value)?;
                            }
                        }
                    }
                }
            }

            Ok(())
        }
    };
}

/// The type of collection being serialized.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CollectionType {
    Array,
    Map,

    /// Tags write headers differently, see `SerializeTupleVariant::serialize_field`.
    Tag,
}

/// An internal struct for serializing collections.
///
/// Not to be used externally, only exposed as part of the [Serializer] type.
#[doc(hidden)]
pub struct CollectionSerializer<'a, W, C: Canonicalization> {
    serializer: &'a mut Serializer<W, C>,
    collection_type: CollectionType,

    /// None if the collection is indefinite length. Canonical serialization will ignore this.
    length: Option<usize>,

    /// First serialized value is the tag header, use this to track whether said tag header has
    /// been written yet. Only relevant for tag collections.
    tag_written: bool,

    #[cfg(feature = "std")]
    cache_keys: Vec<Vec<u8>>,
    #[cfg(feature = "std")]
    cache_values: Vec<Vec<u8>>,
}

impl<'a, W: Write, C: Canonicalization> CollectionSerializer<'a, W, C>
where
    W::Error: core::fmt::Debug,
{
    #[inline(always)]
    fn new(
        serializer: &'a mut Serializer<W, C>,
        collection_type: CollectionType,
        length: Option<usize>,
    ) -> Result<Self, Error<W::Error>> {
        let mut collection_serializer = Self {
            serializer,
            collection_type,
            length,
            tag_written: !matches!(collection_type, CollectionType::Tag),
            #[cfg(feature = "std")]
            cache_keys: Vec::new(),
            #[cfg(feature = "std")]
            cache_values: Vec::new(),
        };

        if !C::IS_CANONICAL {
            collection_serializer.push_header(length)?;
        }

        Ok(collection_serializer)
    }

    #[inline(always)]
    fn push_header(&mut self, length: Option<usize>) -> Result<(), Error<W::Error>> {
        match self.collection_type {
            CollectionType::Array => Ok(self.serializer.encoder.push(Header::Array(length))?),
            CollectionType::Map => Ok(self.serializer.encoder.push(Header::Map(length))?),
            // tag headers are always written directly in SerializeTupleVariant::serialize_field
            // as they don't contain a potentially unknown length
            CollectionType::Tag => Ok(()),
        }
    }

    #[inline(always)]
    fn inline_serialize_key<U: ?Sized + ser::Serialize>(
        &mut self,
        key: &U,
    ) -> Result<(), Error<W::Error>> {
        match C::IS_CANONICAL {
            false => key.serialize(&mut *self.serializer),

            #[cfg(not(feature = "std"))]
            true => unreachable!(),

            #[cfg(feature = "std")]
            true => {
                // use to_vec_small, we expect keys to be smaller than values
                let key_bytes =
                    to_vec_small::<_, C>(key).map_err(|e| Error::Value(e.to_string()))?;
                self.cache_keys.push(key_bytes);
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn inline_serialize_value<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Error<W::Error>> {
        match C::IS_CANONICAL {
            false => value.serialize(&mut *self.serializer),

            #[cfg(not(feature = "std"))]
            true => unreachable!(),

            #[cfg(feature = "std")]
            true => {
                // use to_vec_canonical, we expect values to be bigger than keys
                let value_bytes =
                    to_vec_canonical::<_, C>(value).map_err(|e| Error::Value(e.to_string()))?;
                self.cache_values.push(value_bytes);
                Ok(())
            }
        }
    }
}

impl<'a, W: Write, C: Canonicalization> ser::SerializeSeq for CollectionSerializer<'a, W, C>
where
    W::Error: core::fmt::Debug,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_element<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Self::Error> {
        self.inline_serialize_value(value)
    }

    end!();
}

impl<'a, W: Write, C: Canonicalization> ser::SerializeTuple for CollectionSerializer<'a, W, C>
where
    W::Error: core::fmt::Debug,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_element<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Self::Error> {
        self.inline_serialize_value(value)
    }

    end!();
}

impl<'a, W: Write, C: Canonicalization> ser::SerializeTupleStruct for CollectionSerializer<'a, W, C>
where
    W::Error: core::fmt::Debug,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Self::Error> {
        self.inline_serialize_value(value)
    }

    end!();
}

impl<'a, W: Write, C: Canonicalization> ser::SerializeTupleVariant
    for CollectionSerializer<'a, W, C>
where
    W::Error: core::fmt::Debug,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Self::Error> {
        if self.tag_written {
            // untagged tuples are CollectionType::Array to skip writing the tag header
            return self.inline_serialize_value(value);
        }

        self.tag_written = true;
        match value.serialize(crate::tag::Serializer) {
            // safe to push the tag header when canonicalizing
            Ok(x) => Ok(self.serializer.encoder.push(Header::Tag(x))?),
            _ => Err(Error::Value("expected tag".into())),
        }
    }

    end!();
}

impl<'a, W: Write, C: Canonicalization> ser::SerializeMap for CollectionSerializer<'a, W, C>
where
    W::Error: core::fmt::Debug,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_key<U: ?Sized + ser::Serialize>(&mut self, key: &U) -> Result<(), Self::Error> {
        self.inline_serialize_key(key)
    }

    #[inline]
    fn serialize_value<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Self::Error> {
        self.inline_serialize_value(value)
    }

    end_map!();
}

impl<'a, W: Write, C: Canonicalization> ser::SerializeStruct for CollectionSerializer<'a, W, C>
where
    W::Error: core::fmt::Debug,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        key: &'static str,
        value: &U,
    ) -> Result<(), Self::Error> {
        self.inline_serialize_key(key)?;
        self.inline_serialize_value(value)?;
        Ok(())
    }

    end_map!();
}

impl<'a, W: Write, C: Canonicalization> ser::SerializeStructVariant
    for CollectionSerializer<'a, W, C>
where
    W::Error: core::fmt::Debug,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        key: &'static str,
        value: &U,
    ) -> Result<(), Self::Error> {
        self.inline_serialize_key(key)?;
        self.inline_serialize_value(value)?;
        Ok(())
    }

    end_map!();
}

/// Internal method for serializing individual keys/values for canonicalization.
///
/// Uses a smaller Vec buffer, as we are expecting smaller keys/values to be serialized.
///
/// We use a very small buffer (2 words) to ensure it's cheap to initialize the Vec. Often the keys
/// and values may only be a couple bytes long such as with integer values.
#[cfg(feature = "std")]
#[inline]
fn to_vec_small<T: ?Sized + ser::Serialize, C: Canonicalization>(
    value: &T,
) -> Result<Vec<u8>, Error<std::io::Error>> {
    let mut buffer = Vec::with_capacity(32);
    let mut serializer: Serializer<_, C> = Serializer::new(&mut buffer);
    value.serialize(&mut serializer)?;
    Ok(buffer)
}

/// Serializes as CBOR into `Vec<u8>`.
///
/// # Example
/// ```rust
/// use ciborium::to_vec;
///
/// #[derive(serde::Serialize)]
/// struct Example {
///     a: u64,
///     aa: u64,
///     b: u64,
/// }
///
/// let example = Example { a: 42, aa: 420, b: 4200 };
/// let bytes = to_vec(&example).unwrap();
///
/// assert_eq!(hex::encode(&bytes), "a36161182a6261611901a46162191068");
/// ```
#[cfg(feature = "std")]
#[inline]
pub fn to_vec<T: ?Sized + ser::Serialize>(value: &T) -> Result<Vec<u8>, Error<std::io::Error>> {
    let mut buffer = Vec::with_capacity(128);
    let mut serializer: Serializer<_, NoCanonicalization> = Serializer::new(&mut buffer);
    value.serialize(&mut serializer)?;
    Ok(buffer)
}

/// Canonically serializes as CBOR into `Vec<u8>` with a specified [CanonicalizationScheme].
///
/// # Example
/// ```rust
/// use ciborium::to_vec_canonical;
/// use ciborium::canonical::Rfc8949;
///
/// #[derive(serde::Serialize)]
/// struct Example {
///     a: u64,
///     aa: u64,
///     b: u64,
/// }
///
/// let example = Example { a: 42, aa: 420, b: 4200 };
/// let bytes = to_vec_canonical::<_, Rfc8949>(&example).unwrap();
///
/// assert_eq!(hex::encode(&bytes), "a36161182a61621910686261611901a4");
/// ```
#[cfg(feature = "std")]
#[inline]
pub fn to_vec_canonical<T: ?Sized + ser::Serialize, C: Canonicalization>(
    value: &T,
) -> Result<Vec<u8>, Error<std::io::Error>> {
    let mut buffer = Vec::with_capacity(128);
    let mut serializer: Serializer<_, C> = Serializer::new(&mut buffer);
    value.serialize(&mut serializer)?;
    Ok(buffer)
}

/// Serializes as CBOR into a type with [`impl ciborium_io::Write`](ciborium_io::Write)
///
/// # Example
/// ```rust
/// use ciborium::into_writer;
///
/// #[derive(serde::Serialize)]
/// struct Example {
///     a: u64,
///     aa: u64,
///     b: u64,
/// }
///
/// let example = Example { a: 42, aa: 420, b: 4200 };
///
/// let mut bytes = Vec::new();
/// into_writer(&example, &mut bytes).unwrap();
///
/// assert_eq!(hex::encode(&bytes), "a36161182a6261611901a46162191068");
/// ```
#[inline]
pub fn into_writer<T: ?Sized + ser::Serialize, W: Write>(
    value: &T,
    writer: W,
) -> Result<(), Error<W::Error>>
where
    W::Error: core::fmt::Debug,
{
    let mut encoder = Serializer::<_, NoCanonicalization>::from(writer);
    value.serialize(&mut encoder)
}

/// Canonically serializes as CBOR into a type with [`impl ciborium_io::Write`](ciborium_io::Write)
///
/// This will sort map keys in output according to a specified [CanonicalizationScheme].
///
/// # Example
/// ```rust
/// use ciborium::into_writer_canonical;
/// use ciborium::canonical::Rfc8949;
///
/// #[derive(serde::Serialize)]
/// struct Example {
///     a: u64,
///     aa: u64,
///     b: u64,
/// }
///
/// let example = Example { a: 42, aa: 420, b: 4200 };
///
/// let mut bytes = Vec::new();
/// into_writer_canonical::<_, _, Rfc8949>(&example, &mut bytes).unwrap();
///
/// assert_eq!(hex::encode(&bytes), "a36161182a61621910686261611901a4");
/// ```
#[cfg(feature = "std")]
#[inline]
pub fn into_writer_canonical<T: ?Sized + ser::Serialize, W: Write, C: Canonicalization>(
    value: &T,
    writer: W,
) -> Result<(), Error<W::Error>>
where
    W::Error: core::fmt::Debug,
{
    let mut encoder: Serializer<W, C> = Serializer::new(writer);
    value.serialize(&mut encoder)
}
