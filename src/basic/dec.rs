use super::*;
use crate::io::Read;

use core::convert::TryInto;

#[derive(Debug)]
pub enum Error<T> {
    /// An error occurred while reading bytes
    ///
    /// Contains the underlying error reaturned while reading.
    Io(T),

    /// An error occurred while parsing bytes
    ///
    /// Contains the offset into the stream where the syntax error occurred.
    Syntax(usize),
}

impl<T> From<T> for Error<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Io(value)
    }
}

pub trait Itemizer<T> {
    type Error;

    fn pull(&mut self) -> Result<T, Self::Error>;
    fn push(&mut self, item: T);
}

pub struct Decoder<R: Read> {
    reader: R,
    offset: usize,
    buffer: Option<Title>,
}

impl<R: Read> From<R> for Decoder<R> {
    #[inline]
    fn from(value: R) -> Self {
        Self {
            reader: value,
            offset: 0,
            buffer: None,
        }
    }
}

impl<R: Read> Read for Decoder<R> {
    type Error = R::Error;

    #[inline]
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        assert!(self.buffer.is_none());
        self.reader.read_exact(data)?;
        self.offset += data.len();
        Ok(())
    }
}

impl<R: Read> Itemizer<Title> for Decoder<R> {
    type Error = Error<R::Error>;

    #[inline]
    fn pull(&mut self) -> Result<Title, Self::Error> {
        if let Some(title) = self.buffer.take() {
            self.offset += title.1.as_ref().len() + 1;
            return Ok(title);
        }

        let mut prefix = [0u8; 1];
        self.read_exact(&mut prefix[..])?;

        let major = match prefix[0] >> 5 {
            0 => Major::Positive,
            1 => Major::Negative,
            2 => Major::Bytes,
            3 => Major::Text,
            4 => Major::Array,
            5 => Major::Map,
            6 => Major::Tag,
            7 => Major::Other,
            _ => unreachable!(),
        };

        let mut minor = match prefix[0] & 0b00011111 {
            x if x < 24 => Minor::This(x),
            24 => Minor::Next1([0; 1]),
            25 => Minor::Next2([0; 2]),
            26 => Minor::Next4([0; 4]),
            27 => Minor::Next8([0; 8]),
            31 => Minor::More,
            _ => return Err(Error::Syntax(self.offset - 1)),
        };

        self.read_exact(minor.as_mut())?;
        Ok(Title(major, minor))
    }

    #[inline]
    fn push(&mut self, item: Title) {
        assert!(self.buffer.is_none());
        self.buffer = Some(item);
        self.offset -= item.1.as_ref().len() + 1;
    }
}

impl<R: Read> Itemizer<Header> for Decoder<R> {
    type Error = Error<R::Error>;

    #[inline]
    fn pull(&mut self) -> Result<Header, Self::Error> {
        let offset = self.offset;
        let title: Title = self.pull()?;
        title.try_into().map_err(|_| Error::Syntax(offset))
    }

    #[inline]
    fn push(&mut self, item: Header) {
        self.push(Title::from(item))
    }
}

impl<R: Read> Decoder<R> {
    #[inline]
    pub fn offset(&mut self) -> usize {
        self.offset
    }

    #[inline]
    pub fn bytes<'a>(
        &'a mut self,
        len: Option<usize>,
        buf: &'a mut [u8],
    ) -> Segments<'a, R, Bytes> {
        self.push(Header::Bytes(len));
        Segments::new(self, buf, |header| match header {
            Header::Bytes(len) => Ok(len),
            _ => Err(()),
        })
    }

    #[inline]
    pub fn text<'a>(&'a mut self, len: Option<usize>, buf: &'a mut [u8]) -> Segments<'a, R, Text> {
        self.push(Header::Text(len));
        Segments::new(self, buf, |header| match header {
            Header::Text(len) => Ok(len),
            _ => Err(()),
        })
    }
}
