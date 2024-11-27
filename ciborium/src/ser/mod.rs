// SPDX-License-Identifier: Apache-2.0

//! Serde serialization support for CBOR

mod error;

pub use error::Error;

use alloc::string::ToString;
use ciborium_io::Write;
use ciborium_ll::*;
use serde::{ser, Serialize as _};

/// Which canonicalization scheme to use for CBOR serialization.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CanonicalizationScheme {
    /// No canonicalization, don't sort map keys. Faster and reduces allocations.
    None,

    /// Sort map keys in output according to [RFC 7049]'s deterministic encoding spec.
    ///
    /// Also aligns with [RFC 8949 4.2.3]'s backwards compatibility sort order.
    ///
    /// Uses length-first map key ordering. Eg. `["a", "b", "aa"]`.
    #[cfg(feature = "std")]
    Rfc7049,

    /// Sort map keys in output according to [RFC 8949]'s deterministic encoding spec.
    ///
    /// Uses bytewise lexicographic map key ordering. Eg. `["a", "aa", "b"]`.
    #[cfg(feature = "std")]
    Rfc8049,
}

impl CanonicalizationScheme {
    /// Does this canonicalization scheme require sorting of keys.
    pub fn is_sorting(&self) -> bool {
        #[cfg(feature = "std")] {
            matches!(self, Self::Rfc7049 | Self::Rfc8049)
        }

        #[cfg(not(feature = "std"))] {
            false
        }
    }
}

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
///     let mut serializer = Serializer::new(&mut buffer, ciborium::ser::CanonicalizationScheme::Rfc8049);
///     example.serialize(&mut serializer).unwrap();
///     assert_eq!(hex::encode(&buffer), "a36161182a61621910686261611901a4");
/// }
///
/// #[cfg(not(feature = "std"))] {
///     let mut serializer = Serializer::from(&mut buffer);  // uses no canonicalization
///     example.serialize(&mut serializer).unwrap();
///     assert_eq!(hex::encode(&buffer), "a36161182a6261611901a46162191068");
/// }
/// ```
pub struct Serializer<W> {
    encoder: Encoder<W>,

    /// Whether to canonically sort map keys in output according a particular
    /// [CanonicalizationScheme] map key sort ordering.
    canonicalization: CanonicalizationScheme,
}

impl<W: Write> Serializer<W> {
    /// Create a new CBOR serializer.
    ///
    /// `canonicalization` can be used to change the [CanonicalizationScheme] used for sorting
    /// output map and struct keys to ensure deterministic outputs.
    pub fn new(encoder: impl Into<Encoder<W>>, canonicalization: CanonicalizationScheme) -> Self {
        Self {
            encoder: encoder.into(),
            canonicalization,
        }
    }
}

impl<W: Write> From<W> for Serializer<W> {
    #[inline]
    fn from(writer: W) -> Self {
        Self {
            encoder: writer.into(),
            #[cfg(feature = "std")]
            canonicalization: CanonicalizationScheme::None,
        }
    }
}

impl<W: Write> From<Encoder<W>> for Serializer<W> {
    #[inline]
    fn from(writer: Encoder<W>) -> Self {
        Self {
            encoder: writer,
            #[cfg(feature = "std")]
            canonicalization: CanonicalizationScheme::None,
        }
    }
}

impl<'a, W: Write> ser::Serializer for &'a mut Serializer<W>
where
    W::Error: core::fmt::Debug,
{
    type Ok = ();
    type Error = Error<W::Error>;

    type SerializeSeq = CollectionSerializer<'a, W>;
    type SerializeTuple = CollectionSerializer<'a, W>;
    type SerializeTupleStruct = CollectionSerializer<'a, W>;
    type SerializeTupleVariant = CollectionSerializer<'a, W>;
    type SerializeMap = CollectionSerializer<'a, W>;
    type SerializeStruct = CollectionSerializer<'a, W>;
    type SerializeStructVariant = CollectionSerializer<'a, W>;

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
            ("@@TAG@@", "@@TAGGED@@") => CollectionSerializer::new(self, CollectionType::Tag, Some(length)),

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
        #[inline]
        fn end(mut self) -> Result<(), Self::Error> {
            if self.serializer.canonicalization.is_sorting() {
                // Canonical serialization holds back writing headers, as it doesn't allow
                // indefinite length structs. This allows us to always compute the length.
                self.push_header(Some(self.cache_values.len()))?;

                for value in self.cache_values.iter() {
                    self.serializer.encoder.write_all(&value)?;
                }
            } else if self.length.is_none() {
                // Not canonical and no length => indefinite length break.
                self.serializer.encoder.push(Header::Break)?;
            }

            Ok(())
        }
    };
}

macro_rules! end_map {
    () => {
        #[inline]
        fn end(mut self) -> Result<(), Self::Error> {
            if self.serializer.canonicalization.is_sorting() {
                // Canonical serialization holds back writing headers, as it doesn't allow
                // indefinite length structs. This allows us to always compute the length.
                self.push_header(Some(self.cache_keys.len()))?;
            }

            // Sort our cached output and write it to the encoder.
            match self.serializer.canonicalization {
                CanonicalizationScheme::None => {}
                #[cfg(feature = "std")]
                CanonicalizationScheme::Rfc8049 => {
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
                #[cfg(feature = "std")]
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

            if self.length.is_none() && !self.serializer.canonicalization.is_sorting() {
                // Not canonical and no length => indefinite length break.
                self.serializer.encoder.push(Header::Break)?;
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
pub struct CollectionSerializer<'a, W> {
    serializer: &'a mut Serializer<W>,
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

impl<'a, W: Write> CollectionSerializer<'a, W>
where
    W::Error: core::fmt::Debug,
{
    #[inline]
    fn new(
        serializer: &'a mut Serializer<W>,
        collection_type: CollectionType,
        length: Option<usize>,
    ) -> Result<Self, Error<W::Error>> {
        let mut collection_serializer = Self {
            serializer,
            collection_type,
            length,
            tag_written: false,
            #[cfg(feature = "std")]
            cache_keys: Vec::with_capacity(0),
            #[cfg(feature = "std")]
            cache_values: Vec::with_capacity(0),
        };

        if !collection_serializer.serializer.canonicalization.is_sorting() {
            collection_serializer.push_header(length)?;
        }

        Ok(collection_serializer)
    }

    #[inline]
    fn push_header(&mut self, length: Option<usize>) -> Result<(), Error<W::Error>> {
        match self.collection_type {
            CollectionType::Array => {
                Ok(self.serializer.encoder.push(Header::Array(length))?)
            }
            CollectionType::Map => {
                Ok(self.serializer.encoder.push(Header::Map(length))?)
            }
            // tag headers are always written directly in SerializeTupleVariant::serialize_field
            // as they don't contain a potentially unknown length
            CollectionType::Tag => Ok(()),
        }
    }
}

impl<'a, W: Write> ser::SerializeSeq for CollectionSerializer<'a, W>
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
        #[cfg(feature = "std")]
        if self.serializer.canonicalization.is_sorting() {
            let value_bytes = to_vec_small(value, self.serializer.canonicalization)
                .map_err(|e| Error::Value(e.to_string()))?;
            self.cache_values.push(value_bytes);
            return Ok(());
        }

        value.serialize(&mut *self.serializer)
    }

    end!();
}

impl<'a, W: Write> ser::SerializeTuple for CollectionSerializer<'a, W>
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
        #[cfg(feature = "std")]
        if self.serializer.canonicalization.is_sorting() {
            let value_bytes = to_vec_small(value, self.serializer.canonicalization)
                .map_err(|e| Error::Value(e.to_string()))?;
            self.cache_values.push(value_bytes);
            return Ok(());
        }

        value.serialize(&mut *self.serializer)
    }

    end!();
}

impl<'a, W: Write> ser::SerializeTupleStruct for CollectionSerializer<'a, W>
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
        #[cfg(feature = "std")]
        if self.serializer.canonicalization.is_sorting() {
            let value_bytes = to_vec_small(value, self.serializer.canonicalization)
                .map_err(|e| Error::Value(e.to_string()))?;
            self.cache_values.push(value_bytes);
            return Ok(());
        }

        value.serialize(&mut *self.serializer)
    }

    end!();
}

impl<'a, W: Write> ser::SerializeTupleVariant for CollectionSerializer<'a, W>
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
        if self.tag_written || !matches!(self.collection_type, CollectionType::Tag) {
            #[cfg(feature = "std")]
            if self.serializer.canonicalization.is_sorting() {
                let value_bytes = to_vec_small(value, self.serializer.canonicalization)
                    .map_err(|e| Error::Value(e.to_string()))?;
                self.cache_values.push(value_bytes);
                return Ok(());
            }

            return value.serialize(&mut *self.serializer);
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

impl<'a, W: Write> ser::SerializeMap for CollectionSerializer<'a, W>
where
    W::Error: core::fmt::Debug,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_key<U: ?Sized + ser::Serialize>(&mut self, key: &U) -> Result<(), Self::Error> {
        if self.serializer.canonicalization.is_sorting() {
            let key_bytes = to_vec_small(key, self.serializer.canonicalization)
                .map_err(|e| Error::Value(e.to_string()))?;
            self.cache_keys.push(key_bytes);
            return Ok(());
        }

        key.serialize(&mut *self.serializer)
    }

    #[inline]
    fn serialize_value<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U,
    ) -> Result<(), Self::Error> {
        #[cfg(feature = "std")]
        if self.serializer.canonicalization.is_sorting() {
            let value_bytes = to_vec_small(value, self.serializer.canonicalization)
                .map_err(|e| Error::Value(e.to_string()))?;
            self.cache_values.push(value_bytes);
            return Ok(());
        }

        value.serialize(&mut *self.serializer)
    }

    end_map!();
}

impl<'a, W: Write> ser::SerializeStruct for CollectionSerializer<'a, W>
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
        #[cfg(feature = "std")]
        if self.serializer.canonicalization.is_sorting() {
            let key_bytes = to_vec_small(key, self.serializer.canonicalization)
                .map_err(|e| Error::Value(e.to_string()))?;
            self.cache_keys.push(key_bytes);
            let value_bytes = to_vec_small(value, self.serializer.canonicalization)
                .map_err(|e| Error::Value(e.to_string()))?;
            self.cache_values.push(value_bytes);
            return Ok(());
        }

        key.serialize(&mut *self.serializer)?;
        value.serialize(&mut *self.serializer)?;
        Ok(())
    }

    end_map!();
}

impl<'a, W: Write> ser::SerializeStructVariant for CollectionSerializer<'a, W>
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
        #[cfg(feature = "std")]
        if self.serializer.canonicalization.is_sorting() {
            let key_bytes = to_vec_small(key, self.serializer.canonicalization)
                .map_err(|e| Error::Value(e.to_string()))?;
            self.cache_keys.push(key_bytes);
            let value_bytes = to_vec_small(value, self.serializer.canonicalization)
                .map_err(|e| Error::Value(e.to_string()))?;
            self.cache_values.push(value_bytes);
            return Ok(());
        }

        key.serialize(&mut *self.serializer)?;
        value.serialize(&mut *self.serializer)?;
        Ok(())
    }

    end_map!();
}

/// Internal method for serializing individual keys/values for canonicalization.
///
/// Uses a smaller Vec buffer, as we are expecting smaller keys/values to be serialized.
///
/// We use a very small buffer (2 words) to ensure it's cheap to initialize the Vec. Often the keys
/// and values may only be a couple bytes long such as with integer values. Some kind of type length
/// hint could help in the future, or perhaps using a smallvec crate too.
#[inline]
pub fn to_vec_small<T: ?Sized + ser::Serialize>(
    value: &T,
    canonicalization_scheme: CanonicalizationScheme,
) -> Result<Vec<u8>, Error<std::io::Error>> {
    let mut buffer = std::vec::Vec::with_capacity(16);
    let mut serializer = Serializer::new(&mut buffer, canonicalization_scheme);
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
    let mut buffer = std::vec::Vec::with_capacity(128);
    let mut serializer = Serializer::new(&mut buffer, CanonicalizationScheme::None);
    value.serialize(&mut serializer)?;
    Ok(buffer)
}

/// Canonically serializes as CBOR into `Vec<u8>` with a specified [CanonicalizationScheme].
///
/// # Example
/// ```rust
/// use ciborium::to_vec_canonical;
/// use ciborium::ser::CanonicalizationScheme;
///
/// #[derive(serde::Serialize)]
/// struct Example {
///     a: u64,
///     aa: u64,
///     b: u64,
/// }
///
/// let example = Example { a: 42, aa: 420, b: 4200 };
/// let bytes = to_vec_canonical(&example, CanonicalizationScheme::Rfc8049).unwrap();
///
/// assert_eq!(hex::encode(&bytes), "a36161182a61621910686261611901a4");
/// ```
#[cfg(feature = "std")]
#[inline]
pub fn to_vec_canonical<T: ?Sized + ser::Serialize>(
    value: &T,
    scheme: CanonicalizationScheme,
) -> Result<Vec<u8>, Error<std::io::Error>> {
    let mut buffer = std::vec::Vec::with_capacity(128);
    let mut serializer = Serializer::new(&mut buffer, scheme);
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
    let mut encoder = Serializer::from(writer);
    value.serialize(&mut encoder)
}

/// Canonically serializes as CBOR into a type with [`impl ciborium_io::Write`](ciborium_io::Write)
///
/// This will sort map keys in output according to a specified [CanonicalizationScheme].
///
/// # Example
/// ```rust
/// use ciborium::into_writer_canonical;
/// use ciborium::ser::CanonicalizationScheme;
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
/// into_writer_canonical(&example, &mut bytes, CanonicalizationScheme::Rfc8049).unwrap();
///
/// assert_eq!(hex::encode(&bytes), "a36161182a61621910686261611901a4");
/// ```
#[cfg(feature = "std")]
#[inline]
pub fn into_writer_canonical<T: ?Sized + ser::Serialize, W: Write>(
    value: &T,
    writer: W,
    scheme: CanonicalizationScheme,
) -> Result<(), Error<W::Error>>
where
    W::Error: core::fmt::Debug,
{
    let mut encoder = Serializer::new(writer, scheme);
    value.serialize(&mut encoder)
}
