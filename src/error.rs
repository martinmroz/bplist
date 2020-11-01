//
// Copyright 2020 bplist Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

/// Binary property list serialization and deserialization error.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Constructed from serialization and deserialization errors.
    Message(String),
    /// The bplist magic number and version marker is missing or invalid.
    MissingOrInvalidHeader,
    /// The offset table used determine the location of objects in the object table is missing or invalid.
    MissingOrInvalidOffsetTable,
    /// The trailer with the metadata necessary to interpret the offset table and object table is missing or invalid.
    MissingOrInvalidTrailer,
    /// The version number in the header is not one of the versions supported by this library.
    UnsupportedVersion,
    /// Encountered a reference to an object not in the offset table.
    InvalidObjectReference,
    /// Encountered an offset to an object not in the object table.
    InvalidOffsetToObject,
    /// Encountered a marker byte for an object format not supported by this library.
    InvalidOrUnsupportedObjectFormat,
    /// The root object in the trailer is not in the offset table.
    InvalidRootObject,
    /// The root object is not an array or dictionary.
    RootObjectNotArrayOrDictionary,
    /// The current object was expected to be a valid boolean, but parsing it failed.
    ExpectedBool,
    /// The current object was expected to be a valid fill byte, but parsing it failed.
    ExpectedFill,
    /// The current object was expected to be a valid 8-bit unsigned integer, but parsing it failed.
    ExpectedUInt8,
    /// The current object was expected to be a valid 16-bit unsigned integer, but parsing it failed.
    ExpectedUInt16,
    /// The current object was expected to be a valid 32-bit unsigned integer, but parsing it failed.
    ExpectedUInt32,
    /// The current object was expected to be a valid 64-bit unsigned integer, but parsing it failed.
    ExpectedSInt64,
    /// The current object was expected to be a valid 32-bit single-precision floating point value.
    ExpectedFloat32,
    /// The current object was expected to be a valid 64-bit double-precision floating point value.
    ExpectedFloat64,
    /// The current object was expected to be a valid array, but parsing it failed.
    ExpectedArray,
    /// The current object was expected to be a valid date object, but parsing it failed.
    ExpectedDate,
    /// The current object was expected to be a valid data buffer, but parsing it failed.
    ExpectedData,
    /// The current object was expected to be a valid ASCII string, but parsing it failed.
    ExpectedAsciiString,
    /// The current object was expected to be a valid UTF-16 string, but parsing it failed.
    ExpectedUtf16String,
    /// The current object was expected to be a valid UID blob, but parsing it failed.
    ExpectedUid,
    /// The current object was expected to be a valid dictionary, but parsing it failed.
    ExpectedDictionary,
    /// Binary property lists are directed acyclic graphs and objects cannot reference each other.
    CycleDetected,
    /// Prematurely reached the end of the file.
    Eof,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) =>
                formatter.write_str(msg),
            Error::MissingOrInvalidHeader =>
                formatter.write_str("missing or invalid bplist header"),
            Error::MissingOrInvalidOffsetTable =>
                formatter.write_str("missing or invalid bplist offset table"),
            Error::MissingOrInvalidTrailer =>
                formatter.write_str("missing or invalid bplist trailer"),
            Error::UnsupportedVersion =>
                formatter.write_str("document is not a version (0,0) bplist"),
            Error::InvalidObjectReference =>
                formatter.write_str("invalid reference to object not in offset table"),
            Error::InvalidOffsetToObject =>
                formatter.write_str("invalid offset to element in offset table"),
            Error::InvalidOrUnsupportedObjectFormat =>
                formatter.write_str("invalird or unsupported object format encountered"),
            Error::InvalidRootObject =>
                formatter.write_str("invalid root object in document metadata"),
            Error::RootObjectNotArrayOrDictionary =>
                formatter.write_str("root object is not an array or dictionary"),
            Error::ExpectedBool =>
                formatter.write_str("expected boolean"),
            Error::ExpectedFill =>
                formatter.write_str("expected fill unit type"),
            Error::ExpectedUInt8 =>
                formatter.write_str("expected 8-bit unsigned integer"),
            Error::ExpectedUInt16 =>
                formatter.write_str("expected 16-bit unsigned integer"),
            Error::ExpectedUInt32 =>
                formatter.write_str("expected 32-bit unsigned integer"),
            Error::ExpectedSInt64 =>
                formatter.write_str("expected 64-bit signed integer"),
            Error::ExpectedFloat32 =>
                formatter.write_str("expected 32-bit single-precision floating point value"),
            Error::ExpectedFloat64 =>
                formatter.write_str("expected 64-bit double-precision floating point value"),
            Error::ExpectedArray =>
                formatter.write_str("expected array of object references"),
            Error::ExpectedDate =>
                formatter.write_str("expected CFAbsoluteTime value"),
            Error::ExpectedData =>
                formatter.write_str("expected data"),
            Error::ExpectedAsciiString =>
                formatter.write_str("expected ASCII string"),
            Error::ExpectedUtf16String =>
                formatter.write_str("expected UTF-16 string"),
            Error::ExpectedUid =>
                formatter.write_str("expected UID value"),
            Error::ExpectedDictionary =>
                formatter.write_str("expected dictionary"),
            Error::CycleDetected =>
                formatter.write_str("cycle detected"),
            Error::Eof =>
                formatter.write_str("unexpected end of input"),
        }
    }
}

impl std::error::Error for Error {}
