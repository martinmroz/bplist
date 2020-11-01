//
// Copyright 2020 bplist Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

//! Serde does not provide a Date or Time type.
//! As such, the binary plist Date type is mapped onto a custom Date structure.
//! This is achieved by having it represented during deserialization as a structure with
//! a special name and field, similar to the way the TOML crate approaches it.

use serde::de;

use std::fmt;

use crate::object::Date;

/// Name of the Date structure.
pub const STRUCT_NAME: &str = "$__bplist_private_Date";

/// Name of the field in the structure.
pub const STRUCT_FIELD: &str = "$__bplist_private_Date_absolute_time";

/// Custom deserializer for the Date pseudo-structure.
impl<'de> de::Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct DateVisitor;

        // Process the structure as a map.
        impl<'de> de::Visitor<'de> for DateVisitor {
            type Value = Date;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a date")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Date, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let value = visitor.next_key::<DateKey>()?;
                if value.is_none() {
                    return Err(de::Error::custom("date key not found"));
                }
                let date_from_f64: DateFromF64 = visitor.next_value()?;
                Ok(date_from_f64.value)
            }
        }

        // Deserialize the Date structure with the special name and field.
        deserializer.deserialize_struct(
            STRUCT_NAME,
            &[STRUCT_FIELD],
            DateVisitor
        )
    }
}

struct DateKey;

/// Deserializes the custom date struct field.
impl<'de> de::Deserialize<'de> for DateKey {
    fn deserialize<D>(deserializer: D) -> Result<DateKey, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a valid date field")
            }

            fn visit_str<E>(self, s: &str) -> Result<(), E>
            where
                E: de::Error,
            {
                if s == STRUCT_FIELD {
                    Ok(())
                } else {
                    Err(de::Error::custom("expected field with custom name"))
                }
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)?;
        Ok(DateKey)
    }
}

pub struct DateFromF64 {
    pub value: Date,
}

impl<'de> de::Deserialize<'de> for DateFromF64 {
    fn deserialize<D>(deserializer: D) -> Result<DateFromF64, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = DateFromF64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("absolute time offset from core data epoch")
            }

            fn visit_f64<E>(self, v: f64) -> Result<DateFromF64, E>
            where
                E: de::Error,
            {
                Ok(DateFromF64 {
                    value: Date {
                        absolute_time: v.into()
                    }
                })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}
