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
pub const STRUCT_NAME: &str = "__bplist_private_CF$UID";

/// Name of the field in the structure.
pub const STRUCT_FIELD: &str = "__bplist_private_CF$UID_value";

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
                formatter.write_str("a uid")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Uid, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let value = visitor.next_key::<UidKey>()?;
                if value.is_none() {
                    return Err(de::Error::custom("uid key not found"));
                }
                let uid_from_u64: UidFromU64 = visitor.next_value()?;
                Ok(uid_from_u64.value)
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
                formatter.write_str("a valid uid field")
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

pub struct UidFromU64 {
    pub value: Uid,
}

impl<'de> de::Deserialize<'de> for UidFromU64 {
    fn deserialize<D>(deserializer: D) -> Result<UidFromU64, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = UidFromU64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("uid value")
            }

            fn visit_u64<E>(self, v: u64) -> Result<UidFromU64, E>
            where
                E: de::Error,
            {
                Ok(UidFromU64 {
                    value: Uid(v)
                })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}
