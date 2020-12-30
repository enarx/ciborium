// SPDX-License-Identifier: Apache-2.0

use core::cmp::{Ord, Ordering, PartialOrd};
use core::convert::TryFrom;
use core::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

/// An error that occurred while converting between floating point values
#[derive(Debug)]
pub struct TryFromFloatError(());

/// An abstract floating point value
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Float(f64);

impl From<f32> for Float {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value.into())
    }
}

impl From<f64> for Float {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl TryFrom<Float> for f32 {
    type Error = TryFromFloatError;

    #[inline]
    fn try_from(value: Float) -> Result<Self, Self::Error> {
        let n32 = value.0 as f32;

        if (n32 as f64).to_bits() == value.0.to_bits() {
            return Ok(n32);
        }

        Err(TryFromFloatError(()))
    }
}

impl From<Float> for f64 {
    #[inline]
    fn from(value: Float) -> Self {
        value.0
    }
}

impl Hash for Float {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.to_bits().hash(hasher)
    }
}

impl Eq for Float {}
impl PartialEq for Float {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        self.0.to_bits() == rhs.0.to_bits()
    }
}

impl Ord for Float {
    #[inline]
    fn cmp(&self, rhs: &Self) -> Ordering {
        match (self.0.is_nan(), rhs.0.is_nan()) {
            (false, false) => self.0.partial_cmp(&rhs.0).unwrap(),
            (false, true) => Ordering::Less,
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
        }
    }
}

impl PartialOrd for Float {
    #[inline]
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
