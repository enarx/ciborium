use super::*;

use crate::io::Read;

use core::marker::PhantomData;

pub trait Parser: Default {
    type Item: ?Sized;
    type Error;

    fn parse<'a>(&mut self, bytes: &'a mut [u8]) -> Result<&'a Self::Item, Self::Error>;

    fn saved(&self) -> usize {
        0
    }
}

#[derive(Default)]
pub struct Bytes(());

impl Parser for Bytes {
    type Item = [u8];
    type Error = core::convert::Infallible;

    fn parse<'a>(&mut self, bytes: &'a mut [u8]) -> Result<&'a [u8], Self::Error> {
        Ok(bytes)
    }
}

#[derive(Default)]
pub struct Text {
    stored: usize,
    buffer: [u8; 3],
}

impl Parser for Text {
    type Item = str;
    type Error = core::str::Utf8Error;

    fn parse<'a>(&mut self, bytes: &'a mut [u8]) -> Result<&'a str, Self::Error> {
        // If we cannot advance, return nothing.
        if bytes.len() <= self.stored {
            return Ok("");
        }

        // Copy previously invalid data into place.
        bytes[..self.stored].clone_from_slice(&self.buffer[..self.stored]);

        Ok(match core::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
                let valid_len = e.valid_up_to();
                let invalid_len = bytes.len() - valid_len;

                // If the size of the invalid UTF-8 is large enough to hold
                // all valid UTF-8 characters, we have a syntax error.
                if invalid_len > self.buffer.len() {
                    return Err(e);
                }

                // Otherwise, store the invalid bytes for the next read cycle.
                self.buffer[..invalid_len].clone_from_slice(&bytes[valid_len..]);
                self.stored = invalid_len;

                // Decode the valid part of the string.
                core::str::from_utf8(&bytes[..valid_len]).unwrap()
            }
        })
    }

    fn saved(&self) -> usize {
        self.stored
    }
}

pub struct Segment<'a, R: Read, P: Parser> {
    reader: &'a mut Decoder<R>,
    buffer: &'a mut [u8],
    unread: usize,
    offset: usize,
    parser: P,
}

impl<'a, R: Read, P: Parser> Segment<'a, R, P> {
    #[inline]
    pub fn next(&mut self) -> Result<Option<&P::Item>, Error<R::Error>> {
        use core::cmp::min;

        let prev = self.parser.saved();
        match self.unread {
            0 if prev == 0 => return Ok(None),
            0 => return Err(Error::Syntax(self.offset)),
            _ => (),
        }

        // Determine how many bytes to read.
        let size = min(self.buffer.len(), prev + self.unread);
        let full = &mut self.buffer[..size];
        let next = &mut full[min(size, prev)..];

        // Read additional bytes.
        self.reader.read_exact(next)?;
        self.unread -= next.len();

        self.parser
            .parse(full)
            .or(Err(Error::Syntax(self.offset)))
            .map(Some)
    }
}

pub struct Segments<'a, R: Read, P: Parser> {
    reader: &'a mut Decoder<R>,
    buffer: Option<&'a mut [u8]>,
    nested: usize,
    parser: PhantomData<P>,
    unwrap: fn(Header) -> Result<Option<usize>, ()>,
}

impl<'a, R: Read, P: Parser> Segments<'a, R, P> {
    #[inline]
    pub(crate) fn new(
        decoder: &'a mut Decoder<R>,
        buffer: &'a mut [u8],
        unwrap: fn(Header) -> Result<Option<usize>, ()>,
    ) -> Self {
        Self {
            reader: decoder,
            buffer: Some(buffer),
            nested: 0,
            parser: PhantomData,
            unwrap,
        }
    }

    #[inline]
    pub fn next(&mut self) -> Result<Option<Segment<R, P>>, Error<R::Error>> {
        while self.buffer.is_some() {
            let offset = self.reader.offset();
            match self.reader.pull(false)? {
                Header::Break if self.nested == 1 => return Ok(None),
                Header::Break if self.nested > 1 => self.nested -= 1,
                header => match (self.unwrap)(header) {
                    Err(..) => return Err(Error::Syntax(offset)),
                    Ok(None) => self.nested += 1,
                    Ok(Some(len)) => {
                        let buffer = match self.nested {
                            0 => self.buffer.take().unwrap(),
                            _ => self.buffer.as_mut().unwrap(),
                        };

                        return Ok(Some(Segment {
                            reader: self.reader,
                            buffer,
                            unread: len,
                            offset,
                            parser: P::default(),
                        }));
                    }
                },
            }
        }

        Ok(None)
    }
}
