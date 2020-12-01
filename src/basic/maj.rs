// SPDX-License-Identifier: Apache-2.0

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Major {
    Positive,
    Negative,
    Bytes,
    Text,
    Array,
    Map,
    Tag,
    Other,
}
