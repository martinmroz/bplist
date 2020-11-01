//
// Copyright 2020 bplist Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

//! # The bplist object model.
//!
//! The bplist format version 00 supports the following object kinds:
//!
//! 1. Boolean.
//! 2. Integers, up to 64 bits long.
//! 3. Real, single- and double-precision.
//! 4. Data.
//! 5. Date.
//! 6. String.
//! 7. Uid.
//! 8. Array.
//! 9. Dictionary.
//!
//! # References
//!
//! 1. https://github.com/opensource-apple/CF/blob/master/ForFoundationOnly.h
//! 2. https://opensource.apple.com/source/CF/CF-855.17/CFBinaryPList.c

/// A date structure roughly equivalent to an `NSDate`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Date {
    /// A double-precision 64-bit offset, in seconds, from the Core Data Epoch.
    /// This is defined as 1 January 2001, 00:00:00 UTC.
    pub absolute_time: ordered_float::OrderedFloat<f64>,
}

/// A UID structure treating the contents as an opaque big-endian blob.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Uid {
    /// A blob of identifier data.
    pub data: Vec<u8>,
}

/// Represents any valid bplist object.
///
/// See the `bplist::object` module documentation for usage examples.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Object {
    /// Represents a bplist boolean, like `NSNumber`.
    Boolean(bool),

    /// Represents a bplist integral value of up to 64 bits, like `NSNumber`.
    Integer(i64),

    /// Represents a bplist floating-point value, like `NSNumber`.
    ///
    /// Unlike normal `f64` values, these values have a defined order, implementing
    /// `Ord`, `Eq` and `Hash` in addition to `PartialOrd` and `PartialEq`.
    /// This allows them to be used as keys and values in Dictionaries.
    Real(ordered_float::OrderedFloat<f64>),

    /// Represents a bplist data instance, like `NSData`.
    Data(Vec<u8>),

    /// Represents a bplist date, like `NSDate`.
    ///
    /// Dates are encoded as `CFAbsoluteTime` values. This is a double-precision 64-bit
    /// offset, in seconds, from the Core Data Epoch, defined as 1 January 2001, 00:00:00 UTC.
    /// As serde does not have a built-in date type, this is treated as a custom type and
    /// deserialized as a structure.
    Date(Date),

    /// Represents a bplist UID value.
    ///
    /// These opaque data blobs are not decoded beyond collecting the bytes into a `Vec`.
    /// This is a custom type and is deserialized as a structure.
    Uid(Uid),

    /// Represents a bplist string, like `NSString`.
    String(String),

    /// Represents a bplist array of objects, like `NSArray<id>`.
    Array(Vec<Object>),

    /// Represents a bplist dictionary, like `NSDictionary<id,id>`.
    ///
    /// The dictionary is backed by a `BTreeMap` meaning that objects have a defined order
    /// however that is not necessarily going to be the order in which they are
    /// encountered during parsing and therefore may not round-trip cleanly.
    Dictionary(std::collections::BTreeMap<Object, Object>),
}

mod de;
