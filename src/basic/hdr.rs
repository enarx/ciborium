use super::*;

use core::convert::TryFrom;

use half::f16;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Header {
    Positive(u64),
    Negative(u64),
    Float(f64),
    Simple(u8),
    Tag(u64),
    Break,

    Bytes(Option<usize>),
    Text(Option<usize>),
    Array(Option<usize>),
    Map(Option<usize>),
}

impl TryFrom<Title> for Header {
    type Error = InvalidError;

    fn try_from(title: Title) -> Result<Self, Self::Error> {
        let opt = |minor| {
            Some(match minor {
                Minor::This(x) => x.into(),
                Minor::Next1(x) => u8::from_be_bytes(x).into(),
                Minor::Next2(x) => u16::from_be_bytes(x).into(),
                Minor::Next4(x) => u32::from_be_bytes(x).into(),
                Minor::Next8(x) => u64::from_be_bytes(x),
                Minor::More => return None,
            })
        };

        let int = |m| opt(m).ok_or(InvalidError(()));

        let len = |m| {
            opt(m)
                .map(usize::try_from)
                .transpose()
                .or(Err(InvalidError(())))
        };

        Ok(match title {
            Title(Major::Positive, minor) => Self::Positive(int(minor)?),
            Title(Major::Negative, minor) => Self::Negative(int(minor)?),
            Title(Major::Bytes, minor) => Self::Bytes(len(minor)?),
            Title(Major::Text, minor) => Self::Text(len(minor)?),
            Title(Major::Array, minor) => Self::Array(len(minor)?),
            Title(Major::Map, minor) => Self::Map(len(minor)?),
            Title(Major::Tag, minor) => Self::Tag(int(minor)?),

            Title(Major::Other, Minor::More) => Self::Break,
            Title(Major::Other, Minor::This(x)) => Self::Simple(x),
            Title(Major::Other, Minor::Next1(x)) => Self::Simple(x[0]),
            Title(Major::Other, Minor::Next2(x)) => Self::Float(f16::from_be_bytes(x).into()),
            Title(Major::Other, Minor::Next4(x)) => Self::Float(f32::from_be_bytes(x).into()),
            Title(Major::Other, Minor::Next8(x)) => Self::Float(f64::from_be_bytes(x)),
        })
    }
}

impl From<Header> for Title {
    fn from(header: Header) -> Self {
        let int = |i: u64| match i {
            x if x <= 23 => Minor::This(i as u8),
            x if x <= core::u8::MAX as u64 => Minor::Next1([i as u8]),
            x if x <= core::u16::MAX as u64 => Minor::Next2((i as u16).to_be_bytes()),
            x if x <= core::u32::MAX as u64 => Minor::Next4((i as u32).to_be_bytes()),
            x => Minor::Next8(x.to_be_bytes()),
        };

        let len = |l: Option<usize>| l.map(|x| int(x as u64)).unwrap_or(Minor::More);

        match header {
            Header::Positive(x) => Title(Major::Positive, int(x)),
            Header::Negative(x) => Title(Major::Negative, int(x)),
            Header::Bytes(x) => Title(Major::Bytes, len(x)),
            Header::Text(x) => Title(Major::Text, len(x)),
            Header::Array(x) => Title(Major::Array, len(x)),
            Header::Map(x) => Title(Major::Map, len(x)),
            Header::Tag(x) => Title(Major::Tag, int(x)),

            Header::Break => Title(Major::Other, Minor::More),

            Header::Simple(x) => match x {
                x @ 0..=23 => Title(Major::Other, Minor::This(x)),
                x => Title(Major::Other, Minor::Next1([x])),
            },

            Header::Float(n64) => {
                let n16 = half::f16::from_f64(n64);
                let n32 = n64 as f32;

                Title(
                    Major::Other,
                    if f64::from(n16).to_bits() == n64.to_bits() {
                        Minor::Next2(n16.to_be_bytes())
                    } else if f64::from(n32).to_bits() == n64.to_bits() {
                        Minor::Next4(n32.to_be_bytes())
                    } else {
                        Minor::Next8(n64.to_be_bytes())
                    },
                )
            }
        }
    }
}
