use super::*;
use crate::io::Write;

pub struct Encoder<W: Write>(W);

impl<W: Write> From<W> for Encoder<W> {
    #[inline]
    fn from(value: W) -> Self {
        Self(value)
    }
}

impl<W: Write> Write for Encoder<W> {
    type Error = W::Error;

    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.0.write_all(data)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush()
    }
}

impl<W: Write> Encoder<W> {
    #[inline]
    pub fn encode(&mut self, header: Header) -> Result<(), W::Error> {
        let title = Title::from(header);

        let major = match title.0 {
            Major::Positive => 0,
            Major::Negative => 1,
            Major::Bytes => 2,
            Major::Text => 3,
            Major::Array => 4,
            Major::Map => 5,
            Major::Tag => 6,
            Major::Other => 7,
        };

        let minor = match title.1 {
            Minor::This(x) => x,
            Minor::Next1(..) => 24,
            Minor::Next2(..) => 25,
            Minor::Next4(..) => 26,
            Minor::Next8(..) => 27,
            Minor::More => 31,
        };

        self.0.write_all(&[major << 5 | minor])?;
        self.0.write_all(title.1.as_ref())
    }
}
