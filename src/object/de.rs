//
// Copyright 2020 bplist Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use serde::de;
use ordered_float::OrderedFloat;

use std::collections::BTreeMap;
use std::fmt;

use crate::de::{date, uid};
use crate::object::Object;

/// Deserialization of bplist objects into an object model which supports
/// all values that can be losslessly read from and written into a bplist document.
/// This is implemented generically, meaning that substantially any serde format will be
/// able to deserialize into a bplist value. There are two notable exceptions, `Uid`
/// and `Date`. These are implemented as single-entry maps/structs with magic keys,
/// and as such, will only be deserialized from a bplist object.
impl<'de> de::Deserialize<'de> for Object {
    fn deserialize<D>(deserializer: D) -> Result<Object, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ObjectVisitor;

        impl<'de> de::Visitor<'de> for ObjectVisitor {
            type Value = Object;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid bplist object value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Object, E> {
                Ok(Object::Boolean(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Object, E> {
                Ok(Object::Integer(value))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Object, E> 
            where
                E: de::Error {
                if value <= i64::max_value() as u64 {
                    Ok(Object::Integer(value as i64))
                } else {
                    Err(de::Error::custom("u64 value was too large"))
                }
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Object, E> {
                Ok(Object::Real(OrderedFloat::from(value)))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Object, E> {
                Ok(Object::String(value.into()))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Object, E> {
                Ok(Object::String(value))
            }

            #[inline]
            fn visit_bytes<E>(self, value: &[u8]) -> Result<Object, E> {
                Ok(Object::Data(value.into()))
            }

            #[inline]
            fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Object, E> {
                Ok(Object::Data(value))
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Object, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }
                Ok(Object::Array(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Object, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut map = BTreeMap::<Object, Object>::new();

                // Re-mapped types without Serde are identified by a special key.
                let mut key = visitor.next_key()?;

                // These do not map to Dictionaries.
                if let Some(Object::String(ref x @ _ )) = key {
                    if x == date::STRUCT_FIELD {
                        let date_value: date::DateFromF64 = visitor.next_value()?;
                        return Ok(Object::Date(date_value.value));
                    } else if x == uid::STRUCT_FIELD {
                        let uid_value: uid::UidFromU64 = visitor.next_value()?;
                        return Ok(Object::Uid(uid_value.value));
                    }
                }

                // Process all key-value pairs checking for duplicates.
                while let Some(k) = key {
                    if map.contains_key(&k) {
                        let msg = format!("duplicate key: `{:?}`", k);
                        return Err(de::Error::custom(msg));
                    } else {
                        let v = visitor.next_value()?;
                        map.insert(k, v);
                        key = visitor.next_key()?;
                    }
                }

                Ok(Object::Dictionary(map))
            }
        }

        deserializer.deserialize_any(ObjectVisitor)
    }
}
