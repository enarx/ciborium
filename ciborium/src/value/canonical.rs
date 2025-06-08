// SPDX-License-Identifier: Apache-2.0

use alloc::{boxed::Box, string::ToString, vec::Vec};
use ciborium_io::Write;
use core::cmp::Ordering;
use serde::{de, ser};

use crate::value::Value;

/// Manually serialize values to compare them.
fn serialized_canonical_cmp(v1: &Value, v2: &Value) -> Ordering {
    // There is an optimization to be done here, but it would take a lot more code
    // and using mixing keys, Arrays or Maps as CanonicalValue is probably not the
    // best use of this type as it is meant mainly to be used as keys.

    let mut bytes1 = Vec::new();
    let _ = crate::ser::into_writer(v1, &mut bytes1);
    let mut bytes2 = Vec::new();
    let _ = crate::ser::into_writer(v2, &mut bytes2);

    match bytes1.len().cmp(&bytes2.len()) {
        Ordering::Equal => bytes1.cmp(&bytes2),
        x => x,
    }
}

/// Compares two values uses canonical comparison, as defined in both
/// RFC 7049 Section 3.9 (regarding key sorting) and RFC 8949 4.2.3 (as errata).
///
/// In short, the comparison follow the following rules:
///   - If two keys have different lengths, the shorter one sorts earlier;
///   - If two keys have the same length, the one with the lower value in
///     (byte-wise) lexical order sorts earlier.
///
/// This specific comparison allows Maps and sorting that respect these two rules.
pub fn cmp_value(v1: &Value, v2: &Value) -> Ordering {
    use Value::*;

    match (v1, v2) {
        (Integer(i), Integer(o)) => {
            // Because of the first rule above, two numbers might be in a different
            // order than regular i128 comparison. For example, 10 < -1 in
            // canonical ordering, since 10 serializes to `0x0a` and -1 to `0x20`,
            // and -1 < -1000 because of their lengths.
            i.canonical_cmp(o)
        }
        (Text(s), Text(o)) => match s.len().cmp(&o.len()) {
            Ordering::Equal => s.cmp(o),
            x => x,
        },
        (Bool(s), Bool(o)) => s.cmp(o),
        (Null, Null) => Ordering::Equal,
        (Tag(t, v), Tag(ot, ov)) => match Value::from(*t).partial_cmp(&Value::from(*ot)) {
            Some(Ordering::Equal) | None => match v.partial_cmp(ov) {
                Some(x) => x,
                None => serialized_canonical_cmp(v1, v2),
            },
            Some(x) => x,
        },
        (_, _) => serialized_canonical_cmp(v1, v2),
    }
}

/// A CBOR Value that impl Ord and Eq to allow sorting of values as defined in both
/// RFC 7049 Section 3.9 (regarding key sorting) and RFC 8949 4.2.3 (as errata).
///
/// Since a regular [Value] can be
#[derive(Clone, Debug)]
pub struct CanonicalValue(Value);

impl PartialEq for CanonicalValue {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for CanonicalValue {}

impl From<Value> for CanonicalValue {
    fn from(v: Value) -> Self {
        Self(v)
    }
}

impl From<CanonicalValue> for Value {
    fn from(v: CanonicalValue) -> Self {
        v.0
    }
}

impl ser::Serialize for CanonicalValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for CanonicalValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Value::deserialize(deserializer).map(Into::into)
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Value::deserialize_in_place(deserializer, &mut place.0)
    }
}

impl Ord for CanonicalValue {
    fn cmp(&self, other: &Self) -> Ordering {
        cmp_value(&self.0, &other.0)
    }
}

impl PartialOrd for CanonicalValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Recursively convert a Value to its canonical form as defined in RFC 8949 "core deterministic encoding requirements".
pub fn canonical_value(value: Value) -> Value {
    match value {
        Value::Map(entries) => {
            let mut canonical_entries: Vec<(Value, Value)> = entries
                .into_iter()
                .map(|(k, v)| (canonical_value(k), canonical_value(v)))
                .collect();

            // Sort entries based on the canonical comparison of their keys.
            // cmp_value (defined in this file) implements RFC 8949 key sorting.
            canonical_entries.sort_by(|(k1, _), (k2, _)| cmp_value(k1, k2));

            Value::Map(canonical_entries)
        }
        Value::Array(elements) => {
            let canonical_elements: Vec<Value> =
                elements.into_iter().map(canonical_value).collect();
            Value::Array(canonical_elements)
        }
        Value::Tag(tag, inner_value) => {
            // The tag itself is a u64; its representation is handled by the serializer.
            // The inner value must be in canonical form.
            Value::Tag(tag, Box::new(canonical_value(*inner_value)))
        }
        // Other Value variants (Integer, Bytes, Text, Bool, Null, Float)
        // are considered "canonical" in their structure.
        _ => value,
    }
}

/// Serializes an object as CBOR into a writer using RFC 8949 Deterministic Encoding.
#[inline]
pub fn canonical_into_writer<T: ?Sized + ser::Serialize, W: Write>(
    value: &T,
    writer: W,
) -> Result<(), crate::ser::Error<W::Error>>
where
    W::Error: core::fmt::Debug,
{
    let value =
        Value::serialized(value).map_err(|err| crate::ser::Error::Value(err.to_string()))?;

    let cvalue = canonical_value(value);
    crate::into_writer(&cvalue, writer)
}

/// Serializes an object as CBOR into a new Vec<u8> using RFC 8949 Deterministic Encoding.
#[cfg(feature = "std")]
#[inline]
pub fn canonical_into_vec<T: ?Sized + ser::Serialize>(
    value: &T,
) -> Result<Vec<u8>, crate::ser::Error<<Vec<u8> as ciborium_io::Write>::Error>> {
    let value =
        Value::serialized(value).map_err(|err| crate::ser::Error::Value(err.to_string()))?;

    let cvalue = canonical_value(value);
    crate::into_vec(&cvalue)
}
