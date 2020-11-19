// SPDX-License-Identifier: Apache-2.0

use alloc::vec::Vec;

/// An abstract representation for bytes
#[repr(transparent)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes(Vec<u8>);

impl From<Vec<u8>> for Bytes {
    fn from(value: Vec<u8>) -> Bytes {
        Self(value)
    }
}

impl From<&[u8]> for Bytes {
    fn from(value: &[u8]) -> Bytes {
        Self(value.into())
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(value: Bytes) -> Vec<u8> {
        value.0
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Bytes {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl core::ops::Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Bytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
