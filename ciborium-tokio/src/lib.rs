// SPDX-License-Identifier: Apache-2.0

//! Encoding and decoding for tokio

#![deny(clippy::all)]
#![deny(missing_docs)]
#![deny(clippy::cargo)]

use std::io::ErrorKind;
use std::marker::PhantomData;

use bytes::{Buf, BufMut, BytesMut};
use ciborium_serde::{
    de::{from_reader, Error as DeError},
    ser::{into_writer, Error as SerError},
};
use serde::{de, ser};
use tokio_util::codec;

/// A `tokio_util::codec::Encoder` for CBOR frames
pub struct Encoder<T: ser::Serialize>(PhantomData<T>);

impl<T: ser::Serialize> Default for Encoder<T> {
    #[inline]
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: ser::Serialize> codec::Encoder<&T> for Encoder<T> {
    type Error = SerError<std::io::Error>;

    #[inline]
    fn encode(&mut self, item: &T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        into_writer(item, dst.writer())
    }
}

/// A `tokio_util::codec::Decoder` for CBOR frames
pub struct Decoder<'de, T: de::Deserialize<'de>>(PhantomData<&'de T>);

impl<'de, T: de::Deserialize<'de>> Default for Decoder<'de, T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<'de, T: de::Deserialize<'de>> codec::Decoder for Decoder<'de, T> {
    type Item = T;
    type Error = DeError<std::io::Error>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut bytes: &[u8] = src.as_ref();
        let starting = bytes.len();

        let item: T = match from_reader(&mut bytes) {
            Err(DeError::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Ok(v) => v,
            e => e?,
        };

        let ending = bytes.len();
        src.advance(starting - ending);
        Ok(Some(item))
    }
}

/// A Codec for CBOR frames
pub struct Codec<'de, T: ser::Serialize, U: de::Deserialize<'de>>(Encoder<T>, Decoder<'de, U>);

impl<'de, T: ser::Serialize, U: de::Deserialize<'de>> Default for Codec<'de, T, U> {
    fn default() -> Self {
        Codec(Encoder::default(), Decoder::default())
    }
}

impl<'de, T: ser::Serialize, U: de::Deserialize<'de>> codec::Encoder<&T> for Codec<'de, T, U> {
    type Error = SerError<std::io::Error>;

    #[inline]
    fn encode(&mut self, item: &T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.0.encode(item, dst)
    }
}

impl<'de, T: ser::Serialize, U: de::Deserialize<'de>> codec::Decoder for Codec<'de, T, U> {
    type Item = U;
    type Error = DeError<std::io::Error>;

    #[inline]
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.1.decode(src)
    }
}
