// SPDX-License-Identifier: Apache-2.0

/// The type of the CBOR item
///
/// This structure represents the first three bits of a CBOR item prefix.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Major {
    /// A positive integer
    Positive,

    /// A negative integer
    Negative,

    /// A byte string
    Bytes,

    /// A UTF-8 string
    Text,

    /// An array of CBOR items
    Array,

    /// An array of a pair of CBOR items (i.e. key and value)
    Map,

    /// A CBOR item tag
    Tag,

    /// Other types of a CBOR item
    ///
    /// This includes floating point values, simple constant values - for
    /// example, true, false and null - and the break item used for terminating
    /// items with an indeterminate length.
    Other,
}
