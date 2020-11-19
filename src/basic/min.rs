// SPDX-License-Identifier: Apache-2.0

use super::*;

use core::convert::TryFrom;

use half::f16;

/// Additional CBOR item details
///
/// This type represents both the `Minor` and the `Affix` section of the
/// format as outlined in the chart in this module. The `Minor` section
/// indicates how many following bytes to read. The `Affix` section
/// contains the number of bytes indicated by the `Minor`. Both of these
/// sections are contained in this type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Minor {
    /// The `Minor` section contains the value (`Affix` has zero length)
    Immediate(Immediate),

    /// The `Affix` section contains one byte
    Subsequent1([u8; 1]),

    /// The `Affix` section contains two bytes
    Subsequent2([u8; 2]),

    /// The `Affix` section contains four bytes
    Subsequent4([u8; 4]),

    /// The `Affix` section contains eight bytes
    Subsequent8([u8; 8]),

    /// The `Suffix` length is indeterminate (`Affix` has zero length)
    ///
    /// This case indicates that the `Suffix` contains a stream of CBOR items
    /// or indicates the termination of the stream (i.e. the break value).
    ///
    /// This value is valid for the following `Major` values:
    ///
    ///   * `Major::Bytes` (`Suffix` contains `Major::Bytes` items)
    ///   * `Major::Text` (`Suffix` contains `Major::Text` items)
    ///   * `Major::Array` (`Suffix` contains CBOR items of any type)
    ///   * `Major::Map` (`Suffix` contains CBOR items of any type)
    ///   * `Major::Other` (the break value)
    Indeterminate,
}

impl AsRef<[u8]> for Minor {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Indeterminate => &[],
            Self::Immediate(_) => &[],
            Self::Subsequent1(x) => x.as_ref(),
            Self::Subsequent2(x) => x.as_ref(),
            Self::Subsequent4(x) => x.as_ref(),
            Self::Subsequent8(x) => x.as_ref(),
        }
    }
}

impl AsMut<[u8]> for Minor {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Self::Indeterminate => &mut [],
            Self::Immediate(_) => &mut [],
            Self::Subsequent1(x) => x.as_mut(),
            Self::Subsequent2(x) => x.as_mut(),
            Self::Subsequent4(x) => x.as_mut(),
            Self::Subsequent8(x) => x.as_mut(),
        }
    }
}

impl From<Minor> for Option<u64> {
    #[inline]
    fn from(value: Minor) -> Self {
        Some(match value {
            Minor::Immediate(x) => x.into(),
            Minor::Subsequent1(x) => u8::from_be_bytes(x).into(),
            Minor::Subsequent2(x) => u16::from_be_bytes(x).into(),
            Minor::Subsequent4(x) => u32::from_be_bytes(x).into(),
            Minor::Subsequent8(x) => u64::from_be_bytes(x),
            Minor::Indeterminate => return None,
        })
    }
}

impl TryFrom<Minor> for f32 {
    type Error = InvalidError;

    #[inline]
    #[allow(clippy::float_cmp)]
    fn try_from(value: Minor) -> Result<Self, Self::Error> {
        let n64 = f64::try_from(value)?;
        let n32 = n64 as f32;

        if n32 as f64 == n64 || (n32.is_nan() && n64.is_nan()) {
            Ok(n32)
        } else {
            Err(InvalidError(()))
        }
    }
}

impl TryFrom<Minor> for f64 {
    type Error = InvalidError;

    #[inline]
    fn try_from(value: Minor) -> Result<Self, Self::Error> {
        Ok(match value {
            Minor::Subsequent2(x) => f16::from_be_bytes(x).into(),
            Minor::Subsequent4(x) => f32::from_be_bytes(x).into(),
            Minor::Subsequent8(x) => f64::from_be_bytes(x),
            _ => return Err(InvalidError(())),
        })
    }
}

impl From<u64> for Minor {
    #[inline]
    fn from(value: u64) -> Self {
        if let Ok(value) = Immediate::try_from(value) {
            Self::Immediate(value)
        } else if let Ok(value) = u8::try_from(value) {
            Self::Subsequent1(value.to_be_bytes())
        } else if let Ok(value) = u16::try_from(value) {
            Self::Subsequent2(value.to_be_bytes())
        } else if let Ok(value) = u32::try_from(value) {
            Self::Subsequent4(value.to_be_bytes())
        } else {
            Self::Subsequent8(value.to_be_bytes())
        }
    }
}

impl From<Option<u64>> for Minor {
    #[inline]
    fn from(value: Option<u64>) -> Self {
        match value {
            Some(x) => x.into(),
            None => Self::Indeterminate,
        }
    }
}

impl From<f32> for Minor {
    #[inline]
    #[allow(clippy::float_cmp)]
    fn from(n32: f32) -> Self {
        let n16 = f16::from_f32(n32);

        if n32 == n16.into() || (n32.is_nan() && n16.is_nan()) {
            Self::Subsequent2(n16.to_be_bytes())
        } else {
            Self::Subsequent4(n32.to_be_bytes())
        }
    }
}

impl From<f64> for Minor {
    #[inline]
    #[allow(clippy::float_cmp)]
    fn from(n64: f64) -> Self {
        let n32 = n64 as f32;
        let n16 = f16::from_f64(n64);

        if n64 == n16.into() || (n64.is_nan() && n16.is_nan()) {
            Self::Subsequent2(n16.to_be_bytes())
        } else if n64 == n32.into() || (n64.is_nan() && n32.is_nan()) {
            Self::Subsequent4(n32.to_be_bytes())
        } else {
            Self::Subsequent8(n64.to_be_bytes())
        }
    }
}
