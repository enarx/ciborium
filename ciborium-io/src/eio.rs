use crate::{Read, Write};

/// Wrapper around W: embedded_io::Write implementing ciborium::Write
pub struct EIOWriter<'a, W>(&'a mut W);

impl<'a, W> EIOWriter<'a, W> {
    /// construct EIOWriter for embedded_io::Write
    pub fn from(writer: &'a mut W) -> Self {
        Self(writer)
    }
}

impl<'a, W> Write for EIOWriter<'a, W>
where W: embedded_io::Write
{
    type Error = W::Error;

    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        embedded_io::Write::write_all(self.0, data)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        embedded_io::Write::flush(self.0)
    }
}


/// Wrapper around R: embedded_io::Read implementing ciborium::Read
pub struct EIOReader<'a, R>(&'a mut R);

impl<'a, R> EIOReader<'a, R> {
    /// construct EIOReader for embedded_io::Read
    pub fn from(reader: &'a mut R) -> Self {
        Self(reader)
    }
}

impl<'a, R> Read for EIOReader<'a, R>
where R: embedded_io::Read
{
    type Error = embedded_io::ReadExactError<R::Error>;

    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        embedded_io::Read::read_exact(self.0, data)
    }
}
