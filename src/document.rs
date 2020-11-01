//
// Copyright 2020 bplist Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

//! # The bplist document format.
//!
//! Constants and structures defined in this module represent the various
//! sections of a binary property list document.
//!
//! A bplist document is organized into four segments:
//!
//! 1. Header
//! 2. Object Table
//! 3. Offset Table
//! 4. Trailer
//!
//! # References
//!
//! 1. https://github.com/opensource-apple/CF/blob/master/ForFoundationOnly.h
//! 2. https://opensource.apple.com/source/CF/CF-855.17/CFBinaryPList.c

/// The number of bytes of data required to define a bplist header.
pub const HEADER_SIZE: usize = 8;

/// The bplist magic number ("bplist").
pub const HEADER_MAGIC_NUMBER: &[u8] = &[ 0x62, 0x70, 0x6C, 0x69, 0x73, 0x74 ];

/// The bplist version 00 identifier ("00").
pub const HEADER_VERSION_00: (u8, u8) = (0x30, 0x30);

/// Binary property list header.
///
/// The header is composed of a magic number and a two-byte version marker
/// representing the major and minor version of the serialization format in
/// which the document is encoded.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash)]
pub struct Header {
    /// The bplist version as as two 8-bit values.
    pub version: (u8, u8)
}

/// Binary property list offset table.
///
/// The offset table is a mapping from element identifiers to byte offset from
/// the start of the file at which the object resides.
pub type OffsetTable = Vec<usize>;

/// Binary property list object wire format.
///
/// The bplist00 format is self-describing. Each object consists of a marker byte
/// and zero or more bytes of additional data. The marker byte is comprised of
/// 'tag' bits, which specifes its format, and up to four bits of embedded value data.
/// 
/// # Notes
/// 1. All values are stored in network byte order (big endian).
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum ObjectFormat {
    /// A 1-bit boolean literal value.
    Boolean,
    /// A 'fill' byte, used for padding.
    Fill,
    /// An unsigned 8-bit integer.
    UInt8,
    /// An unsigned 16-bit integer.
    UInt16,
    /// An unsigned 32-bit integer.
    UInt32,
    /// A signed 64-bit integer.
    SInt64,
    /// A single-precision 32-bit floating point value.
    Float32,
    /// A double-precision 64-bit floating point value.
    Float64,
    /// A 64-bit double-precision CFAbsoluteTime value.
    Date,
    /// An arbitrary set of bytes.
    Data,
    /// A 7-bit ASCII string.
    AsciiString,
    /// A 16-bit UTF16 string.
    Utf16String,
    /// A UID used by NSArchiver.
    Uid,
    /// An array.
    Array,
    /// A dictionary.
    Dictionary,
}

impl ObjectFormat {

    /// Compute the bitwise AND of the marker byte and tag mask to obtain the its bits.
    pub fn tag_mask(self) -> u8 {
        use ObjectFormat::*;
        match self {
            Boolean =>
                0b1111_1110,
            Fill | UInt8 | UInt16 | UInt32 | SInt64 | Float32 | Float64 | Date =>
                0b1111_1111,
            Data | AsciiString | Utf16String | Uid | Array | Dictionary =>
                0b1111_0000,
        }
    }

    /// Compute the bitwise AND of the marker byte and the value mask to obtain its value bits.
    pub fn value_mask(self) -> u8 {
        use ObjectFormat::*;
        match self {
            Boolean =>
                0b0000_0001,
            Fill | UInt8 | UInt16 | UInt32 | SInt64 | Float32 | Float64 | Date =>
                0b0000_0000,
            Data | AsciiString | Utf16String | Uid | Array | Dictionary =>
                0b0000_1111,
        }
    }

    /// Uniquely identifies the object format when compared to the tag bits of a marker byte.
    pub fn tag_bits(self) -> u8 {
        use ObjectFormat::*;
        match self {
            Boolean =>
                0b0000_1000,
            Fill =>
                0b0000_1111,
            UInt8 => 
                0b0001_0000,
            UInt16 => 
                0b0001_0001,
            UInt32 =>
                0b0001_0010,
            SInt64 => 
                0b0001_0011,
            Float32 =>
                0b0010_0010,
            Float64 => 
                0b0010_0011,
            Date =>
                0b0011_0011,
            Data =>
                0b0100_0000,
            AsciiString =>
                0b0101_0000,
            Utf16String =>
                0b0110_0000,
            Uid =>
                0b1000_0000,
            Array => 
                0b1010_0000,
            Dictionary =>
                0b1101_0000,
        }
    }

}

/// The number of bytes of data required to define a bplist trailer.
pub const TRAILER_SIZE: usize = 32;

/// The number of unused bytes in the trailer preamble.
pub const TRAILER_PREAMBLE_UNUSED_SIZE: usize = 5;

/// Binary property list trailer.
/// 
/// The trailer contains information necessary to interpret the preceding
/// document, particularly the size of variably-sized offsets and references.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Trailer {
    /// The sort version.
    pub sort_version: u8,
    /// Number of bytes needed for each offset table entry.
    pub offset_table_entry_size: usize,
    /// Number of bytes needed for each object reference in a container.
    pub object_reference_size: usize,
    /// Number of objects encoded in the document.
    pub number_of_objects: usize,
    /// Element id of the root object.
    pub root_object: usize,
    /// Offset into the file denoting the start of the offset table.
    pub offset_table_offset: usize,
}
