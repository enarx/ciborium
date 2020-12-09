// SPDX-License-Identifier: Apache-2.0

use super::*;

use core::convert::TryFrom;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Minor {
    This(u8),
    Next1([u8; 1]),
    Next2([u8; 2]),
    Next4([u8; 4]),
    Next8([u8; 8]),
    More,
}

impl TryFrom<u8> for Minor {
    type Error = InvalidError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value & 0b00011111 {
            x @ 0..=23 => Minor::This(x),
            24 => Minor::Next1([0u8; 1]),
            25 => Minor::Next2([0u8; 2]),
            26 => Minor::Next4([0u8; 4]),
            27 => Minor::Next8([0u8; 8]),
            31 => Minor::More,
            _ => return Err(InvalidError(())),
        })
    }
}

impl From<Minor> for u8 {
    #[inline]
    fn from(value: Minor) -> Self {
        match value {
            Minor::This(x) => x,
            Minor::Next1(..) => 24,
            Minor::Next2(..) => 25,
            Minor::Next4(..) => 26,
            Minor::Next8(..) => 27,
            Minor::More => 31,
        }
    }
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

impl From<Minor> for Option<u64> {
    #[inline]
    fn from(value: Minor) -> Self {
        Some(match value {
            Minor::This(x) => x.into(),
            Minor::Next1(x) => u8::from_be_bytes(x).into(),
            Minor::Next2(x) => u16::from_be_bytes(x).into(),
            Minor::Next4(x) => u32::from_be_bytes(x).into(),
            Minor::Next8(x) => u64::from_be_bytes(x),
            Minor::More => return None,
        })
    }
}

impl TryFrom<Minor> for Option<usize> {
    type Error = InvalidError;

    #[inline]
    fn try_from(value: Minor) -> Result<Self, Self::Error> {
        Option::<u64>::from(value)
            .map(usize::try_from)
            .transpose()
            .or(Err(InvalidError(())))
    }
}

impl From<u64> for Minor {
    #[inline]
    fn from(value: u64) -> Self {
        if value < 24 {
            Self::This(value as u8)
        } else if let Ok(value) = u8::try_from(value) {
            Self::Next1(value.to_be_bytes())
        } else if let Ok(value) = u16::try_from(value) {
            Self::Next2(value.to_be_bytes())
        } else if let Ok(value) = u32::try_from(value) {
            Self::Next4(value.to_be_bytes())
        } else {
            Self::Next8(value.to_be_bytes())
        }
    }
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64",))]
impl From<usize> for Minor {
    #[inline]
    fn from(value: usize) -> Self {
        (value as u64).into()
    }
}

impl From<Option<usize>> for Minor {
    #[inline]
    fn from(value: Option<usize>) -> Self {
        match value {
            Some(x) => x.into(),
            None => Self::More,
        }
    }
}
