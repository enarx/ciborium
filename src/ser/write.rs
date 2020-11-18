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

    /// Writes up to `data.len()` bytes, returning the number of bytes written
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error>;

    /// Writes all bytes from `data` or fails
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Flushes all output
    fn flush(&mut self) -> Result<(), Self::Error>;
}

#[cfg(feature = "std")]
mod std {
    use std::io::*;

    impl<T: Write> super::Write for T {
        type Error = Error;

        #[inline]
        fn write(&mut self, data: &[u8]) -> Result<usize> {
            self.write(data)
        }

        #[inline]
        fn write_all(&mut self, data: &[u8]) -> Result<()> {
            self.write_all(data)
        }

        #[inline]
        fn flush(&mut self) -> Result<()> {
            self.flush()
        }
    }
}

#[cfg(not(feature = "std"))]
mod no_std {
    use alloc::vec::Vec;
    use core::cmp::min;
    use core::mem::replace;

    #[derive(Debug)]
    pub enum Infallible {}

    impl core::fmt::Display for Infallible {
        #[inline]
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl serde::ser::StdError for Infallible {}

    impl super::Write for Vec<u8> {
        type Error = Infallible;

        #[inline]
        fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
            self.extend_from_slice(data);
            Ok(data.len())
        }

        #[inline]
        fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            self.extend_from_slice(data);
            Ok(())
        }

        #[inline]
        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct OutOfSpace;

    impl core::fmt::Display for OutOfSpace {
        #[inline]
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl serde::ser::StdError for OutOfSpace {}

    impl super::Write for &mut [u8] {
        type Error = OutOfSpace;

        #[inline]
        fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
            let len = min(data.len(), self.len());
            let (prefix, suffix) = replace(self, &mut []).split_at_mut(len);
            prefix.copy_from_slice(&data[..len]);
            *self = suffix;
            Ok(len)
        }

        #[inline]
        fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            if data.len() != self.write(data)? {
                Err(OutOfSpace)
            } else {
                Ok(())
            }
        }

        #[inline]
        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl<W: super::Write + ?Sized> super::Write for &mut W {
        type Error = W::Error;

        #[inline]
        fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
            (**self).write(data)
        }

        #[inline]
        fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            (**self).write_all(data)
        }

        #[inline]
        fn flush(&mut self) -> Result<(), Self::Error> {
            (**self).flush()
        }
    }
}
