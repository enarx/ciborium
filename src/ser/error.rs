// SPDX-License-Identifier: Apache-2.0

use alloc::string::{String, ToString};
use core::fmt::{Debug, Display, Formatter, Result};

use serde::ser::{Error as SerError, StdError};

/// An error occurred during serialization
#[derive(Debug)]
pub enum Error<T: 'static + StdError> {
    /// An error occurred while writing bytes
    ///
    /// Contains the underlying error reaturned while writing.
    Io(T),

    /// An error indicating a value that cannot be serialized
    ///
    /// Contains a description of the problem.
    Value(String),
}

impl<T: 'static + StdError> From<T> for Error<T> {
    #[inline]
    fn from(value: T) -> Self {
        Error::Io(value)
    }
}

impl<T: 'static + StdError + Debug> Display for Error<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self)
    }
}

impl<T: 'static + StdError> StdError for Error<T> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Value(_) => None,
        }
    }
}

impl<T: 'static + StdError> SerError for Error<T> {
    fn custom<U: Display>(msg: U) -> Self {
        Error::Value(msg.to_string())
    }
}
