// SPDX-License-Identifier: Apache-2.0

//! Low level CBOR parsing tools
//!
//! This module contains utility types for encoding and decoding items in
//! CBOR. See below for an overview of what a CBOR item looks like on the
//! wire. This module **does not** parse the CBOR item `Suffix`, which
//! typically contains raw bytes, a UTF-8 string or other CBOR items.
//!
//! The most important type in this crate is `Title`, which is the locus
//! for encoding and decoding.
//!
//! # Anatomy of a CBOR Item
//!
//! This is a brief anatomy of a CBOR item on the wire. For more information,
//! see [RFC 7049](https://tools.ietf.org/html/rfc7049).
//!
//! ```text
//! +------------+-----------+
//! |            |           |
//! |   Major    |   Minor   |
//! |  (3bits)   |  (5bits)  |
//! |            |           |
//! +------------+-----------+
//! ^                        ^
//! |                        |
//! +-----+            +-----+
//!       |            |
//!       |            |
//!       +----------------------------+--------------+
//!       |            |               |              |
//!       |   Prefix   |     Affix     |    Suffix    |
//!       |  (1 byte)  |  (0-8 bytes)  |  (0+ bytes)  |
//!       |            |               |              |
//!       +------------+---------------+--------------+
//!
//!       |                            |
//!       +------------+---------------+
//!                    |
//!                    v
//!
//!                  Title
//! ```

mod dec;
mod enc;
mod hdr;
mod seg;

pub use dec::*;
pub use enc::*;
pub use hdr::*;
pub use seg::*;

pub const SIMPLE_FALSE: u8 = 20;
pub const SIMPLE_TRUE: u8 = 21;
pub const SIMPLE_NULL: u8 = 22;
pub const SIMPLE_UNDEFINED: u8 = 23;

pub const TAG_BIGPOS: u64 = 2;
pub const TAG_BIGNEG: u64 = 3;

#[derive(Debug)]
struct InvalidError(());

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Major {
    Positive,
    Negative,
    Bytes,
    Text,
    Array,
    Map,
    Tag,
    Other,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Minor {
    This(u8),
    Next1([u8; 1]),
    Next2([u8; 2]),
    Next4([u8; 4]),
    Next8([u8; 8]),
    More,
}

impl AsRef<[u8]> for Minor {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::More => &[],
            Self::This(..) => &[],
            Self::Next1(x) => x.as_ref(),
            Self::Next2(x) => x.as_ref(),
            Self::Next4(x) => x.as_ref(),
            Self::Next8(x) => x.as_ref(),
        }
    }
}

impl AsMut<[u8]> for Minor {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Self::More => &mut [],
            Self::This(..) => &mut [],
            Self::Next1(x) => x.as_mut(),
            Self::Next2(x) => x.as_mut(),
            Self::Next4(x) => x.as_mut(),
            Self::Next8(x) => x.as_mut(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Title(pub Major, pub Minor);
