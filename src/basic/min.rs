// SPDX-License-Identifier: Apache-2.0

use super::*;

use core::convert::TryFrom;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Minor {
    Immediate(Immediate),
    Subsequent1([u8; 1]),
    Subsequent2([u8; 2]),
    Subsequent4([u8; 4]),
    Subsequent8([u8; 8]),
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
