// SPDX-License-Identifier: Apache-2.0

use super::*;

/// An opaque type representing the first byte of a CBOR item
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Prefix(pub(crate) u8);

impl AsRef<[u8]> for Prefix {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        core::slice::from_ref(&self.0)
    }
}

impl AsMut<[u8]> for Prefix {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        core::slice::from_mut(&mut self.0)
    }
}

impl From<Title> for Prefix {
    #[inline]
    fn from(title: Title) -> Self {
        let major: u8 = (match title.0 {
            Major::Positive => 0,
            Major::Negative => 1,
            Major::Bytes => 2,
            Major::Text => 3,
            Major::Array => 4,
            Major::Map => 5,
            Major::Tag => 6,
            Major::Other => 7,
        }) << 5;

        let minor: u8 = match title.1 {
            Minor::Immediate(x) => x.into(),
            Minor::Subsequent1(_) => 24,
            Minor::Subsequent2(_) => 25,
            Minor::Subsequent4(_) => 26,
            Minor::Subsequent8(_) => 27,
            Minor::Indeterminate => 31,
        };

        Self(major | minor)
    }
}
