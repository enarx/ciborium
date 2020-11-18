// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "std")]
mod std;

#[cfg(not(feature = "std"))]
mod no_std;

/// An error indicating there are no more bytes to read
#[cfg(not(feature = "std"))]
#[derive(Debug)]
pub struct EndOfFile(());

/// An error indicating that the output cannot accept more bytes
#[cfg(not(feature = "std"))]
#[derive(Debug)]
pub struct OutOfSpace(());

/// A trait indicating a type that can read bytes
///
/// Note that this is similar to `std::io::Read`, but simplified for use in a
/// `no_std` context. When the `std` feature is enabled, you will get a
/// blanket implementation for all `std::io::Read` types. Otherwise, you will
/// get an implementation for `&[u8]`.
///
/// You can of course implement this for your own types like always.
pub trait Read {
    /// The error type
    type Error;

    /// Reads exactly `data.len()` bytes or fails
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
}

// SPDX-License-Identifier: Apache-2.0

/// A trait indicating a type that can write bytes
///
/// Note that this is similar to `std::io::Write`, but simplified for use in a
/// `no_std` context. When the `std` feature is enabled, you will get a
/// blanket implementation for all `std::io::Write` types. Otherwise, you will
/// get an implementation for `Vec<u8>` and `&mut [u8]`.
///
/// You can of course implement this for your own types like always.
pub trait Write {
    /// The error type
    type Error;

    /// Writes all bytes from `data` or fails
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Flushes all output
    fn flush(&mut self) -> Result<(), Self::Error>;
}
