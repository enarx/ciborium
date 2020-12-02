// SPDX-License-Identifier: Apache-2.0

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Major {
    Positive,
    Negative,
    Bytes,
    Text,
    Array,
    Map,
    Tag,
    Other,
}

impl From<u8> for Major {
    #[inline]
    fn from(value: u8) -> Self {
        match value >> 5 {
            0 => Major::Positive,
            1 => Major::Negative,
            2 => Major::Bytes,
            3 => Major::Text,
            4 => Major::Array,
            5 => Major::Map,
            6 => Major::Tag,
            7 => Major::Other,
            _ => unreachable!(),
        }
    }
}

impl From<Major> for u8 {
    #[inline]
    fn from(value: Major) -> Self {
        (match value {
            Major::Positive => 0,
            Major::Negative => 1,
            Major::Bytes => 2,
            Major::Text => 3,
            Major::Array => 4,
            Major::Map => 5,
            Major::Tag => 6,
            Major::Other => 7,
        }) << 5
    }
}
