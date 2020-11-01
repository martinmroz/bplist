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
//! 2. Number (Integer or Floating Point).
//! 3. Data (Aribtrary Bytes).
//! 4. Date.
//! 5. String.
//! 6. Uid.
//! 7. Array.
//! 8. Dictionary.
//!
//! # References
//!
//! 1. https://github.com/opensource-apple/CF/blob/master/ForFoundationOnly.h
//! 2. https://opensource.apple.com/source/CF/CF-855.17/CFBinaryPList.c

/// Any value which can be encoded in a binary property list.
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum Object {
    // A boolean literal value.
    Boolean(bool),
    /// A numeric literal value.
    Number(Number),
    /// An array of arbitrary data bytes.
    Data(Vec<u8>),
    /// A date.
    Date(Date),
    /// A UID value.
    Uid(Vec<u8>),
    /// A string.
    String(String),
    /// An array of objects.
    Array(Array),
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
/// A numeric value roughtly equivalent to an `NSNumber`.
pub enum Number {
    /// An integer value.
    Integer(i64),
    /// A single-precision floating-point value.
    Float(f32),
    /// A double-precision floating-point value.
    Double(f64),
}

/// A date structure roughly equivalent to an `NSDate`.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Date {
    /// A double-precision 64-bit offset, in seconds, from the Core Data Epoch.
    /// This is defined as 1 January 2001, 00:00:00 UTC.
    pub offset: f64
}

/// A UID structure treating the contents as an opaque big-endian blob.
#[derive(Clone, PartialEq, Debug)]
pub struct Uid {
    /// A blob of identifier data.
    pub data: Vec<u8>,
}

/// An array of objects roughly equivalent to an `NSArray.
pub type Array = Vec<Object>;
