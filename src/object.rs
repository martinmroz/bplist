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

use ordered_float::OrderedFloat;

use std::collections::HashMap;

/// A date structure roughly equivalent to an `NSDate`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Date {
    /// A double-precision 64-bit offset, in seconds, from the Core Data Epoch.
    /// This is defined as 1 January 2001, 00:00:00 UTC.
    pub absolute_time: OrderedFloat<f64>,
}

/// A UID structure treating the contents as an opaque big-endian blob.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Uid {
    /// A blob of identifier data.
    pub data: Vec<u8>,
}

/// An array of objects roughly equivalent to an `NSArray.
pub type Array = Vec<Object>;

/// An map of Objects to Objects roughly equivalent to an `NSDictionary.
pub type Dictionary = HashMap<Object, Object>;

/// Any value which can be encoded in a binary property list.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Object {
    // A boolean value.
    Boolean(bool),
    /// An integer value.
    Integer(i64),
    /// An floating-point value.
    Real(OrderedFloat<f64>),
    /// An array of arbitrary data bytes.
    Data(Vec<u8>),
    /// A date.
    Date(Date),
    /// A UID value.
    Uid(Uid),
    /// A string.
    String(String),
    /// An array of objects.
    Array(Array),
}
