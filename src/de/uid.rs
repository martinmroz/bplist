//
// Copyright 2020 bplist Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

//! Serde does not provide a data type analogous to a UID.
//! As such, the binary plist UID type is mapped onto a custom UID structure.
//! This is achieved by having it represented during deserialization as a structure with
//! a special name and field, similar to the way the TOML crate approaches Dates.

use serde::de;

use std::fmt;

use crate::object::Uid;

/// Name of the UID structure.
pub const STRUCT_NAME: &str = "$__bplist_private_Uid";

/// Name of the field in the structure.
pub const STRUCT_FIELD: &str = "$__bplist_private_Uid_data";

/// Custom deserializer for the UID pseudo-structure.
impl<'de> de::Deserialize<'de> for Uid {
    fn deserialize<D>(deserializer: D) -> Result<Uid, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct UidVisitor;

        // Process the structure as a map.
        impl<'de> de::Visitor<'de> for UidVisitor {
            type Value = Uid;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a UID")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Uid, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let value = visitor.next_key::<UidKey>()?;
                if value.is_none() {
                    return Err(de::Error::custom("uid key not found"));
                }
                let uid_from_bytes: UidFromBytes = visitor.next_value()?;
                Ok(uid_from_bytes.value)
            }
        }

        // Deserialize the UID structure with the special name and field.
        deserializer.deserialize_struct(
            STRUCT_NAME,
            &[STRUCT_FIELD],
            UidVisitor
        )
    }
}

struct UidKey;

/// Deserializes the custom date struct field.
impl<'de> de::Deserialize<'de> for UidKey {
    fn deserialize<D>(deserializer: D) -> Result<UidKey, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a valid UID field")
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
        Ok(UidKey)
    }
}

pub struct UidFromBytes {
    pub value: Uid,
}

impl<'de> de::Deserialize<'de> for UidFromBytes {
    fn deserialize<D>(deserializer: D) -> Result<UidFromBytes, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = UidFromBytes;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("uid data")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<UidFromBytes, E>
            where
                E: de::Error,
            {
                Ok(UidFromBytes {
                    value: Uid {
                        data: v.into()
                    }
                })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}
