//
// Copyright 2020 bplist Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use nom::{
    IResult,
    bytes::complete::{take, tag},
    combinator::map,
    multi::many_m_n,
    number::complete::be_u8,
    sequence::tuple,
};

use crate::de::parser::utils::be_usize_n;
use crate::document::{
    Header,
    OffsetTable,
    Trailer,
    HEADER_MAGIC_NUMBER,
    TRAILER_PREAMBLE_UNUSED_SIZE,
};

/// Parses a fixed-sized 8-byte header object from the input slice.
pub fn header(input: &[u8]) -> IResult<&[u8], Header> {
    map(
        tuple((
            tag(HEADER_MAGIC_NUMBER),
            be_u8,
            be_u8,
        )), |(
            _,
            version_major,
            version_minor,
        )| Header { 
            version: (version_major, version_minor)
        }
    )(input)
}

/// Returns a parser for an offset table with the specified number of entries,
/// each of the specified size. Offset table entries must be between
/// 1 and 8 bytes long each, inclusive.
pub fn offset_table(
    entries: usize,
    entry_size: usize,
) -> impl Fn(&[u8]) -> IResult<&[u8], OffsetTable> {
    move |input: &[u8]| {
        many_m_n(
            entries,
            entries,
            be_usize_n(entry_size)
        )(input)
    }
}

/// Parses a fixed-sized 32-byte trailer object from the input array.
pub fn trailer(input: &[u8]) -> IResult<&[u8], Trailer> {
    map(
        tuple((
            take(TRAILER_PREAMBLE_UNUSED_SIZE),
            be_u8,
            be_usize_n(1),
            be_usize_n(1),
            be_usize_n(8),
            be_usize_n(8),
            be_usize_n(8),
        )), |(
            _,
            sort_version,
            offset_table_entry_size,
            object_reference_size,
            number_of_objects,
            root_object,
            offset_table_offset,
        )| Trailer {
            sort_version,
            offset_table_entry_size,
            object_reference_size,
            number_of_objects,
            root_object,
            offset_table_offset,
        }
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::document::HEADER_VERSION_00;
    use super::{
        Header,
        Trailer, 
        header,
        offset_table,
        trailer
    };

    #[test]
    fn test_header_bplist00() {
        // 8-byte header for a version 00 bplist.
        let simple_header = &[
            0x62, 0x70, 0x6C, 0x69, 0x73, 0x74,
            0x30,
            0x30,
        ];

        // Parse the header.
        let (residual_data, value) = header(simple_header).unwrap();

        // Validate the entire input was parsed.
        assert_eq!(residual_data.len(), 0);

        // Validate the fields were parsed correctly.
        assert_eq!(value, Header {
            version: HEADER_VERSION_00,
        });
    }

    #[test]
    fn test_offset_table_1bx1() {
        let (input, result) = offset_table(1, 1)(&[0x08]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(result, &[8usize]);
    }

    #[test]
    fn test_offset_table_1bx5() {
        let (input, result) = offset_table(5, 1)(&[0x08, 0x09, 0x10, 0x11, 0x12]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(result, &[8usize, 9usize, 16usize, 17usize, 18usize]);
    }

    #[test]
    fn test_offset_table_8bx5() {
        let (input, result) = offset_table(5, 8)(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12,
        ]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(result, &[8usize, 9usize, 16usize, 17usize, 18usize]);
    }

    #[test]
    fn test_trailer() {
        // 32-byte trailer for a bplist with one object.
        let simple_trailer = &[
            0x00, 0x00, 0x00, 0x00, 0x00, 
            0x00,
            0x01,
            0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09,
        ];

        // Parse the trailer.
        let (residual_data, value) = trailer(simple_trailer).unwrap();

        // Validate the entire input was parsed.
        assert_eq!(residual_data.len(), 0);

        // Validate the fields were parsed correctly.
        assert_eq!(value, Trailer {
            sort_version: 0,
            offset_table_entry_size: 1,
            object_reference_size: 1,
            number_of_objects: 1,
            root_object: 0,
            offset_table_offset: 9
        });
    }
}
