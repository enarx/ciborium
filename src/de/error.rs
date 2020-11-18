// SPDX-License-Identifier: Apache-2.0

use alloc::string::{String, ToString};
use core::fmt::{Debug, Display, Formatter, Result};

use serde::de::{Error as DeError, StdError};

/// An error occurred during deserialization
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

    /// An error occurred while processing a parsed value
    ///
    /// Contains a description of the error that occurred and (optionally)
    /// the offset into the stream indicating the start of the item being
    /// processed when the error occurred.
    Semantic(Option<usize>, String),
}

impl<T> Error<T> {
    /// A helper method for composing a semantic error
    #[inline]
    pub fn semantic(offset: impl Into<Option<usize>>, msg: impl Into<String>) -> Self {
        Self::Semantic(offset.into(), msg.into())
    }
}

impl<T> From<T> for Error<T> {
    #[inline]
    fn from(value: T) -> Self {
        Error::Io(value)
    }
}

impl<T: Debug> Display for Error<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self)
    }
}

impl<T: 'static + StdError> StdError for Error<T> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl<T: 'static + StdError> DeError for Error<T> {
    #[inline]
    fn custom<U: Display>(msg: U) -> Self {
        Self::Semantic(None, msg.to_string())
    }
}
