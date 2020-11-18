// SPDX-License-Identifier: Apache-2.0

//! Low level CBOR parsing tools
//!
//! This module contains utility types for encoding and decoding items in
//! CBOR. See below for an overview of what a CBOR item looks like on the
//! wire. This module **does not** parse the CBOR item `Suffix`, which
//! typically contains raw bytes, a UTF-8 string or other CBOR items.
//!
//! To parse a CBOR item you basically need to do the following:
//!
//! 1. Create a default `Prefix` object.
//! 2. Read data into the `Prefix` object (see `AsMut<[u8]>`).
//! 3. Convert the `Prefix` into a `Title`.
//! 4. Read data into the `Minor` of the `Title` (see `AsMut<[u8]>`).
//!
//! Encoding a CBOR item is likewise simple:
//!
//! 1. Construct a `Title` object containing the data you want.
//! 2. Create a `Prefix` from the `Title`.
//! 3. Write the `Prefix` (see `AsRef<[u8]>`).
//! 4. Write the `Minor` of the `Title` (see `AsRef<[u8]>`).
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
mod pre;
mod tit;

pub use imm::*;
pub use maj::*;
pub use min::*;
pub use pre::*;
pub use tit::*;

/// Validation encountered an invalid value
#[derive(Debug)]
pub struct Invalid(());
