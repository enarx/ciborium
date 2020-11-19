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

mod imm;
mod maj;
mod min;
mod tit;

pub use imm::*;
pub use maj::*;
pub use min::*;
pub use tit::*;

/// Validation encountered an invalid value
#[derive(Debug)]
pub struct InvalidError(());

/// An error that occurred during decoding
#[derive(Debug)]
pub enum DecodeError<T> {
    /// An error occurred reading bytes
    Io(T),

    /// An error occurred parsing bytes
    Invalid,
}

impl<T> From<T> for DecodeError<T> {
    fn from(value: T) -> Self {
        Self::Io(value)
    }
}
