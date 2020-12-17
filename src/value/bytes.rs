// SPDX-License-Identifier: Apache-2.0

use alloc::vec::Vec;

/// An abstract representation for bytes
#[repr(transparent)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes<T>(T);

impl<T> Bytes<T> {
    /// Create a new `Bytes` instance
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> From<T> for Bytes<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl From<Bytes<&[u8]>> for Bytes<Vec<u8>> {
    fn from(value: Bytes<&[u8]>) -> Self {
        Self(value.0.into())
    }
}

impl From<&[u8]> for Bytes<Vec<u8>> {
    fn from(value: &[u8]) -> Self {
        Self(value.into())
    }
}

impl From<Bytes<Vec<u8>>> for Vec<u8> {
    fn from(value: Bytes<Vec<u8>>) -> Self {
        value.0
    }
}

impl<'a> From<Bytes<&'a [u8]>> for &'a [u8] {
    fn from(value: Bytes<&'a [u8]>) -> Self {
        value.0
    }
}

impl From<Bytes<&[u8]>> for Vec<u8> {
    fn from(value: Bytes<&[u8]>) -> Self {
        value.0.into()
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Bytes<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T: AsMut<[u8]>> AsMut<[u8]> for Bytes<T> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl<T: AsRef<[u8]>> core::ops::Deref for Bytes<T> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> core::ops::DerefMut for Bytes<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}
