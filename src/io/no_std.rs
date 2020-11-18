use super::*;

use alloc::vec::Vec;
use core::mem::replace;

impl<R: Read + ?Sized> Read for &mut R {
    type Error = R::Error;

    #[inline]
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        (**self).read_exact(data)
    }
}

impl<W: Write + ?Sized> Write for &mut W {
    type Error = W::Error;

    #[inline]
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        (**self).write_all(data)
    }

    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        (**self).flush()
    }
}

impl Read for &[u8] {
    type Error = EndOfFile;

    #[inline]
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        if data.len() > self.len() {
            return Err(EndOfFile(()));
        }

        let (prefix, suffix) = self.split_at(data.len());
        data.copy_from_slice(prefix);
        *self = suffix;
        Ok(())
    }
}

impl Write for &mut [u8] {
    type Error = OutOfSpace;

    #[inline]
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        if data.len() > self.len() {
            return Err(OutOfSpace(()));
        }

        let (prefix, suffix) = replace(self, &mut []).split_at_mut(data.len());
        prefix.copy_from_slice(data);
        *self = suffix;
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Write for Vec<u8> {
    type Error = core::convert::Infallible;

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
