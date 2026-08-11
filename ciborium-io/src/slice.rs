// SPDX-License-Identifier: Apache-2.0

//! Slice-backed reader utilities.

/// Error returned when attempting to read past the end of a slice.
#[cfg(feature = "std")]
pub type EndOfSliceError = std::io::Error;

/// Error returned when attempting to read past the end of a slice.
#[cfg(not(feature = "std"))]
/// Error returned when attempting to read past the end of a slice.
#[derive(Clone, Debug)]
pub struct EndOfSliceError(());

/// A reader that wraps a byte slice with a lifetime, enabling zero-copy deserialization.
///
/// The unread portion is retained as a subslice, allowing bytes to be consumed or
/// borrowed with a single bounds check.
pub struct SliceReader<'de> {
    remaining: &'de [u8],
}

impl<'de> SliceReader<'de> {
    /// Creates a new `SliceReader` from a byte slice.
    #[inline]
    pub fn new(slice: &'de [u8]) -> Self {
        Self { remaining: slice }
    }

    /// Takes the next `len` bytes without copying them.
    #[inline]
    pub fn take(&mut self, len: usize) -> Option<&'de [u8]> {
        if len > self.remaining.len() {
            return None;
        }

        let (prefix, suffix) = self.remaining.split_at(len);
        self.remaining = suffix;
        Some(prefix)
    }

    /// Returns the number of unread bytes left in the slice.
    #[inline]
    pub fn remaining_len(&self) -> usize {
        self.remaining.len()
    }
}

impl<'de> crate::Read for SliceReader<'de> {
    type Error = EndOfSliceError;

    #[inline]
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), EndOfSliceError> {
        match self.take(data.len()) {
            Some(source) => {
                data.copy_from_slice(source);
                Ok(())
            }
            #[cfg(feature = "std")]
            None => Err(std::io::ErrorKind::UnexpectedEof.into()),
            #[cfg(not(feature = "std"))]
            None => Err(EndOfSliceError(())),
        }
    }
}
