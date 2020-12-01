// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Title(pub Major, pub Minor);

impl Title {
    pub const BREAK: Self = Self(Major::Other, Minor::Indeterminate);

    pub const FALSE: Self = Self(Major::Other, Minor::Immediate(Immediate(20)));
    pub const TRUE: Self = Self(Major::Other, Minor::Immediate(Immediate(21)));
    pub const NULL: Self = Self(Major::Other, Minor::Immediate(Immediate(22)));
    pub const UNDEFINED: Self = Self(Major::Other, Minor::Immediate(Immediate(23)));

    pub const TAG_BIGPOS: Self = Self(Major::Tag, Minor::Immediate(Immediate(2)));
    pub const TAG_BIGNEG: Self = Self(Major::Tag, Minor::Immediate(Immediate(3)));
}
