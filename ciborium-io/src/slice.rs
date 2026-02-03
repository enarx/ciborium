// SPDX-License-Identifier: Apache-2.0

//! Slice-backed reader utilities.

use core::cmp;

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
/// This reader keeps track of the original slice and the current cursor so that other
/// components can create borrowed subslices without copying.
pub struct SliceReader<'de> {
    data: &'de [u8],
    cursor: usize,
}

impl<'de> SliceReader<'de> {
    /// Creates a new `SliceReader` from a byte slice.
    #[inline]
    pub fn new(slice: &'de [u8]) -> Self {
        Self { data: slice, cursor: 0 }
    }

    /// Advances the reader by `len` bytes without copying them anywhere.
    ///
    /// Callers must ensure that `len` does not exceed the remaining slice length.
    #[inline]
    pub fn skip(&mut self, len: usize) {
        debug_assert!(len <= self.remaining_len());
        self.cursor += len;
    }

    /// Returns the number of unread bytes left in the slice.
    #[inline]
    pub fn remaining_len(&self) -> usize {
        self.data.len().saturating_sub(self.cursor)
    }

    /// Returns true if the provided range lies within the source slice.
    #[inline]
    pub fn contains_range(&self, start: usize, len: usize) -> bool {
        start
            .checked_add(len)
            .map_or(false, |end| end <= self.data.len())
    }

    /// Returns the requested subslice without advancing the cursor.
    ///
    /// Callers must ensure the requested range lies within the original data.
    #[inline]
    pub fn slice_at(&self, start: usize, len: usize) -> &'de [u8] {
        &self.data[start..start + len]
    }
}

#[cfg(feature = "std")]
impl<'de> std::io::Read for SliceReader<'de> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining = &self.data[self.cursor..];
        let to_read = cmp::min(buf.len(), remaining.len());
        buf[..to_read].copy_from_slice(&remaining[..to_read]);
        self.cursor += to_read;
        Ok(to_read)
    }
}

#[cfg(not(feature = "std"))]
impl<'de> crate::Read for SliceReader<'de> {
    type Error = EndOfSliceError;

    #[inline]
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), EndOfSliceError> {
        if data.len() > self.remaining_len() {
            return Err(EndOfSliceError(()));
        }
        let end = self.cursor + data.len();
        data.copy_from_slice(&self.data[self.cursor..end]);
        self.cursor = end;
        Ok(())
    }
}
