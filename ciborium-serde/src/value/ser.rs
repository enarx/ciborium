// SPDX-License-Identifier: Apache-2.0

use super::{Error, Value};

use alloc::{vec, vec::Vec};
use core::convert::TryFrom;

use ::serde::ser::{self, SerializeMap as _, SerializeSeq as _};

impl ser::Serialize for Value {
    #[inline]
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Bytes(x) => serializer.serialize_bytes(x),
            Value::Bool(x) => serializer.serialize_bool(*x),
            Value::Text(x) => serializer.serialize_str(x),
            Value::Null => serializer.serialize_unit(),

            Value::Float(x) => {
                if let Ok(x) = f32::try_from(*x) {
                    serializer.serialize_f32(x)
                } else if let Ok(x) = f64::try_from(*x) {
                    serializer.serialize_f64(x)
                } else {
                    unreachable!()
                }
            }

            Value::Integer(x) => {
                if let Ok(x) = u8::try_from(*x) {
                    serializer.serialize_u8(x)
                } else if let Ok(x) = i8::try_from(*x) {
                    serializer.serialize_i8(x)
                } else if let Ok(x) = u16::try_from(*x) {
                    serializer.serialize_u16(x)
                } else if let Ok(x) = i16::try_from(*x) {
                    serializer.serialize_i16(x)
                } else if let Ok(x) = u32::try_from(*x) {
                    serializer.serialize_u32(x)
                } else if let Ok(x) = i32::try_from(*x) {
                    serializer.serialize_i32(x)
                } else if let Ok(x) = u64::try_from(*x) {
                    serializer.serialize_u64(x)
                } else if let Ok(x) = i64::try_from(*x) {
                    serializer.serialize_i64(x)
                } else if let Ok(x) = u128::try_from(*x) {
                    serializer.serialize_u128(x)
                } else if let Ok(x) = i128::try_from(*x) {
                    serializer.serialize_i128(x)
                } else {
                    unreachable!()
                }
            }

            Value::Array(x) => {
                let mut map = serializer.serialize_seq(Some(x.len()))?;

                for v in x {
                    map.serialize_element(v)?;
                }

                map.end()
            }

            Value::Map(x) => {
                let mut map = serializer.serialize_map(Some(x.len()))?;

                for (k, v) in x {
                    map.serialize_entry(k, v)?;
                }

                map.end()
            }
        }
    }
}

macro_rules! mkserialize {
    ($($f:ident($v:ty)),+ $(,)?) => {
        $(
            #[inline]
            fn $f(self, v: $v) -> Result<Self::Ok, Self::Error> {
                Ok(v.into())
            }
        )+
    };
}

struct Serializer<T>(T);

impl ser::Serializer for Serializer<()> {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = Serializer<Vec<Value>>;
    type SerializeTuple = Serializer<Vec<Value>>;
    type SerializeTupleStruct = Serializer<Vec<Value>>;
    type SerializeTupleVariant = Serializer<(&'static str, Vec<Value>)>;
    type SerializeMap = Serializer<(Vec<(Value, Value)>, Option<Value>)>;
    type SerializeStruct = Serializer<Vec<(Value, Value)>>;
    type SerializeStructVariant = Serializer<(&'static str, Vec<(Value, Value)>)>;

    mkserialize! {
        serialize_bool(bool),

        serialize_f32(f32),
        serialize_f64(f64),

        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),

        serialize_char(char),
        serialize_str(&str),
        serialize_bytes(&[u8]),
    }

    #[inline]
    fn serialize_none(self) -> Result<Value, Error> {
        Ok(Value::Null)
    }

    #[inline]
    fn serialize_some<U: ?Sized + ser::Serialize>(self, value: &U) -> Result<Value, Error> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Value, Error> {
        self.serialize_none()
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<Value, Error> {
        Ok(vec![(variant.into(), Value::Null)].into())
    }

    #[inline]
    fn serialize_newtype_struct<U: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        value: &U,
    ) -> Result<Value, Error> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<U: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &U,
    ) -> Result<Value, Error> {
        Ok(vec![(variant.into(), Value::serialized(value)?)].into())
    }

    #[inline]
    fn serialize_seq(self, length: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Ok(Serializer(Vec::with_capacity(length.unwrap_or(0))))
    }

    #[inline]
    fn serialize_tuple(self, length: usize) -> Result<Self::SerializeTuple, Error> {
        self.serialize_seq(Some(length))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        length: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        self.serialize_seq(Some(length))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        length: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Ok(Serializer((variant, Vec::with_capacity(length))))
    }

    #[inline]
    fn serialize_map(self, length: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Ok(Serializer((Vec::with_capacity(length.unwrap_or(0)), None)))
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        length: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        Ok(Serializer(Vec::with_capacity(length)))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        length: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Ok(Serializer((variant, Vec::with_capacity(length))))
    }
}

impl<'a> ser::SerializeSeq for Serializer<Vec<Value>> {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_element<U: ?Sized + ser::Serialize>(&mut self, value: &U) -> Result<(), Error> {
        self.0.push(Value::serialized(&value)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.into())
    }
}

impl<'a> ser::SerializeTuple for Serializer<Vec<Value>> {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_element<U: ?Sized + ser::Serialize>(&mut self, value: &U) -> Result<(), Error> {
        self.0.push(Value::serialized(&value)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.into())
    }
}

impl<'a> ser::SerializeTupleStruct for Serializer<Vec<Value>> {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(&mut self, value: &U) -> Result<(), Error> {
        self.0.push(Value::serialized(&value)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.into())
    }
}

impl<'a> ser::SerializeTupleVariant for Serializer<(&'static str, Vec<Value>)> {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(&mut self, value: &U) -> Result<(), Error> {
        self.0 .1.push(Value::serialized(&value)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(vec![(self.0 .0.into(), self.0 .1.into())].into())
    }
}

impl<'a> ser::SerializeMap for Serializer<(Vec<(Value, Value)>, Option<Value>)> {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_key<U: ?Sized + ser::Serialize>(&mut self, key: &U) -> Result<(), Error> {
        self.0 .1 = Some(Value::serialized(key)?);
        Ok(())
    }

    #[inline]
    fn serialize_value<U: ?Sized + ser::Serialize>(&mut self, value: &U) -> Result<(), Error> {
        let key = self.0 .1.take().unwrap();
        let val = Value::serialized(&value)?;

        self.0 .0.push((key, val));
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0 .0.into())
    }
}

impl<'a> ser::SerializeStruct for Serializer<Vec<(Value, Value)>> {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        key: &'static str,
        value: &U,
    ) -> Result<(), Error> {
        let k = Value::serialized(&key)?;
        let v = Value::serialized(&value)?;
        self.0.push((k, v));
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.into())
    }
}

impl<'a> ser::SerializeStructVariant for Serializer<(&'static str, Vec<(Value, Value)>)> {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_field<U: ?Sized + ser::Serialize>(
        &mut self,
        key: &'static str,
        value: &U,
    ) -> Result<(), Self::Error> {
        let k = Value::serialized(&key)?;
        let v = Value::serialized(&value)?;
        self.0 .1.push((k, v));
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(vec![(self.0 .0.into(), self.0 .1.into())].into())
    }
}

impl Value {
    /// Serializes an object into a `Value`
    #[inline]
    pub fn serialized<T: ?Sized + ser::Serialize>(value: &T) -> Result<Self, Error> {
        value.serialize(Serializer(()))
    }
}
