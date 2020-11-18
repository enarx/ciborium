use super::{Read, Write};

impl<T: std::io::Read> Read for T {
    type Error = std::io::Error;

    #[inline]
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        self.read_exact(data)
    }
}

impl<T: std::io::Write> Write for T {
    type Error = std::io::Error;

    #[inline]
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.write_all(data)
    }

    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush()
    }
}
