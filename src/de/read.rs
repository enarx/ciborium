// SPDX-License-Identifier: Apache-2.0

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

    /// Reads up to `data.len()` bytes, returning the number of bytes read
    fn read(&mut self, data: &mut [u8]) -> Result<usize, Self::Error>;

    /// Reads exactly `data.len()` bytes or fails
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
}

#[cfg(feature = "std")]
mod std {
    impl<T: std::io::Read> super::Read for T {
        type Error = std::io::Error;

        #[inline]
        fn read(&mut self, data: &mut [u8]) -> Result<usize, Self::Error> {
            self.read(data)
        }

        #[inline]
        fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
            self.read_exact(data)
        }
    }
}

#[cfg(not(feature = "std"))]
mod no_std {
    #[derive(Debug)]
    pub struct EndOfFile;

    impl core::fmt::Display for EndOfFile {
        #[inline]
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl serde::de::StdError for EndOfFile {}

    impl super::Read for &[u8] {
        type Error = EndOfFile;

        #[inline]
        fn read(&mut self, data: &mut [u8]) -> Result<usize, Self::Error> {
            let length = core::cmp::min(data.len(), self.len());
            let (prefix, suffix) = self.split_at(length);
            data.copy_from_slice(prefix);
            *self = suffix;
            Ok(length)
        }

        #[inline]
        fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
            let len = data.len();
            if len > self.read(data)? {
                Err(EndOfFile)
            } else {
                Ok(())
            }
        }
    }

    impl<R: super::Read + ?Sized> super::Read for &mut R {
        type Error = R::Error;

        #[inline]
        fn read(&mut self, data: &mut [u8]) -> Result<usize, Self::Error> {
            (**self).read(data)
        }

        #[inline]
        fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
            (**self).read_exact(data)
        }
    }
}
