//
// Copyright 2020 bplist Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use ascii::{AsciiStr, AsAsciiStrError};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::take,
    combinator::{map, map_res, verify},
    multi::many_m_n,
    number::complete::{be_u8, be_u16, be_u32, be_i64, be_f32, be_f64},
    sequence::tuple,
};

use std::convert::TryFrom;
use std::string::FromUtf16Error;

use crate::document::ObjectFormat;
use crate::de::parser::utils::be_usize_n;

/// Returns a parser which consumes a marker conforming to the specified format.
/// On success, the parser yields both the validated format and the encoded value.
/// This allows the function to be used to verify a marker byte is of the specified
/// format and to decode the value contained therein, if any.
fn marker(
    format: ObjectFormat
) -> impl Fn(&[u8]) -> IResult<&[u8], (ObjectFormat, u8)> {
    move |input: &[u8]| {
        map(
            verify(take(1usize), move |b: &[u8]| -> bool {
                (b[0] & format.tag_mask()) == format.tag_bits()
            }),
            move |b: &[u8]| -> (ObjectFormat, u8) {
                (format, b[0] & format.value_mask())
            }
        )(input)
    }
}

/// Parses a marker byte and returns both the object format and encoded value.
pub fn any_marker(input: &[u8]) -> IResult<&[u8], (ObjectFormat, u8)> {
    alt((
        marker(ObjectFormat::Boolean),
        marker(ObjectFormat::Fill),
        marker(ObjectFormat::UInt8),
        marker(ObjectFormat::UInt16),
        marker(ObjectFormat::UInt32),
        marker(ObjectFormat::SInt64),
        marker(ObjectFormat::Float32),
        marker(ObjectFormat::Float64),
        marker(ObjectFormat::Date),
        marker(ObjectFormat::Data),
        marker(ObjectFormat::AsciiString),
        marker(ObjectFormat::Utf16String),
        marker(ObjectFormat::Uid),
        marker(ObjectFormat::Array),
        marker(ObjectFormat::Dictionary),
    ))(input)
}

/// Parses a boolean object with an encoded value bit.
pub fn boolean(input: &[u8]) -> IResult<&[u8], bool> {
    map(
        marker(ObjectFormat::Boolean),
        |(_, value)| value == 1
    )(input)
}

/// Parses a fill object, which is represented as a unit type.
pub fn fill(input: &[u8]) -> IResult<&[u8], ()> {
    map(
        marker(ObjectFormat::Fill),
        |_| ()
    )(input)
}

/// Parses an 8-bit unsigned integer object.
pub fn uint8(input: &[u8]) -> IResult<&[u8], u8> {
    map(
        tuple((
            marker(ObjectFormat::UInt8),
            be_u8,
        )),
        |(_, value)| value
    )(input)
}

/// Parses a 16-bit unsigned integer object.
pub fn uint16(input: &[u8]) -> IResult<&[u8], u16> {
    map(
        tuple((
            marker(ObjectFormat::UInt16),
            be_u16,
        )),
        |(_, value)| value
    )(input)
}

/// Parses a 32-bit unsigned integer object.
pub fn uint32(input: &[u8]) -> IResult<&[u8], u32> {
    map(
        tuple((
            marker(ObjectFormat::UInt32),
            be_u32,
        )),
        |(_, value)| value
    )(input)
}

/// Parses a 64-bit signed integer object.
pub fn sint64(input: &[u8]) -> IResult<&[u8], i64> {
    map(
        tuple((
            marker(ObjectFormat::SInt64),
            be_i64,
        )),
        |(_, value)| value
    )(input)
}

/// Parses a 32-bit single-precision floating point value.
pub fn float32(input: &[u8]) -> IResult<&[u8], f32> {
    map(
        tuple((
            marker(ObjectFormat::Float32),
            be_f32,
        )),
        |(_, value)| value
    )(input)
}

/// Parses a 64-bit double-precision floating point value.
pub fn float64(input: &[u8]) -> IResult<&[u8], f64> {
    map(
        tuple((
            marker(ObjectFormat::Float64),
            be_f64,
        )),
        |(_, value)| value
    )(input)
}

/// Parses a 64-bit double-precision CFTimeInterval date value.
pub fn date(input: &[u8]) -> IResult<&[u8], f64> {
    map(
        tuple((
            marker(ObjectFormat::Date),
            be_f64,
        )),
        |(_, value)| value
    )(input)
}

/// Returns a parser for the length of an object payload.
/// The parameter is the value encoded in the marker byte to which the payload corresponds.
/// If the encoded value is:
///   0b0000_0000 ..= 0b0000_1110:
///     No additional input is consumed and the encoded value represents directly
///     the payload count value.
///   0b0000_1111:
///     An integer object with a 1, 2, 4 or 8 byte payload follows.
///     This object is consumed, interpreted as an unsigned value, and returned.
fn payload_count(
    encoded_value: u8,
) -> impl Fn(&[u8]) -> IResult<&[u8], usize> {
    assert!((encoded_value & 0b1111_0000) == 0, "encoded length must be a 4-bit value");
    move |input: &[u8]| {
        if encoded_value == 0b0000_1111 {
            map_res(
                alt((
                    map(uint8, |value| value as u64),
                    map(uint16, |value| value as u64),
                    map(uint32, |value| value as u64),
                    map(sint64, |value| value as u64),
                )),
                |value| usize::try_from(value)
            )(input)
        } else {
            Ok((input, encoded_value as usize))
        }
    }
}

/// Parses a variable-length data object and returns the corresponding slice of the input.
pub fn data(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, (_, encoded_value)) = marker(ObjectFormat::Data)(input)?;
    let (input, data_length) = payload_count(encoded_value)(input)?;
    take(data_length)(input)
}

/// Parses a variable-length ASCII string object and returns a corresponding borrowed string slice.
/// 
/// # Notes
/// 
/// 1. Validates that the data contained in the object is an ASCII string.
/// 2. This is a zero-copy operation.
pub fn ascii_string(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, (_, encoded_value)) = marker(ObjectFormat::AsciiString)(input)?;
    let (input, char_count) = payload_count(encoded_value)(input)?;
    map_res(
        take(char_count),
        |bytes| -> Result<&str, AsAsciiStrError> {
            AsciiStr::from_ascii(bytes).map(|value| value.as_str())
        }
    )(input)
}

/// Parses a variable-length UTF-16 string object and returns an owned string.
///
/// # Notes
///
/// 1. Validates that the data contained in the object is valid UTF-16.
/// 2. This is not a zero-copy operation.
pub fn utf16_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, (_, encoded_value)) = marker(ObjectFormat::Utf16String)(input)?;
    let (input, char_count) = payload_count(encoded_value)(input)?;
    map_res(
        many_m_n(
            char_count, 
            char_count, 
            be_u16
        ), |code_points| -> Result<String, FromUtf16Error> {
            String::from_utf16(&code_points)
        }
    )(input)
}

/// Parses a variable-length uid object and returns the corresponding slice of the input.
pub fn uid(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, (_, encoded_value)) = marker(ObjectFormat::Uid)(input)?;
    take(encoded_value + 1)(input)
}

/// Returns a parser for an array with the specified-width object references.
///
/// The value returned by the parser is a list of value object references.
pub fn array(
    object_reference_size: usize
) -> impl Fn(&[u8]) -> IResult<&[u8], Vec<usize>> {
    assert!(object_reference_size <= 8, "object references must be up to 8 bytes long");
    move |input: &[u8]| {
        let (input, (_, encoded_value)) = marker(ObjectFormat::Array)(input)?;
        let (input, array_length) = payload_count(encoded_value)(input)?;
        many_m_n(
            array_length, 
            array_length, 
            be_usize_n(object_reference_size)
        )(input)
    }
}

/// Returns a parser for a dictionary with the specified-width key and value references.
///
/// The value returned by the parser is a list of matched key and value object references.
/// In each touple, the key is first and the value is second.
pub fn dictionary(
    object_reference_size: usize
) -> impl Fn(&[u8]) -> IResult<&[u8], Vec<(usize, usize)>> {
    assert!(object_reference_size <= 8, "object references must be up to 8 bytes long");
    move |input: &[u8]| {
        let (input, (_, encoded_value)) = marker(ObjectFormat::Dictionary)(input)?;
        let (input, entry_count) = payload_count(encoded_value)(input)?;
        
        map(
            tuple((
                many_m_n(entry_count, entry_count, be_usize_n(object_reference_size)),
                many_m_n(entry_count, entry_count, be_usize_n(object_reference_size)),
            )), |(
                mut keys, 
                mut values
            )| {
                // Interleave the key and value references
                keys.drain(..)
                    .zip(values.drain(..))
                    .collect::<Vec<(usize, usize)>>()
            }
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_any_marker_valid() {
        let test_markers = &[
            0b0000_1000, // Boolean (false)
            0b0000_1001, // Boolean (true)
            0b0000_1111, // Fill
            0b0001_0000, // UInt8
            0b0001_0001, // UInt16
            0b0001_0010, // UInt32
            0b0001_0011, // SInt64
            0b0010_0010, // Float32
            0b0010_0011, // Float64
            0b0011_0011, // Date
            0b0100_0000, // Data (length 0)
            0b0100_1110, // Data (length 15)
            0b0100_1111, // Data (extended payload)
            0b0101_0000, // ASCII String (length 0)
            0b0101_1110, // ASCII String (length 15)
            0b0101_1111, // ASCII String (extended payload)
            0b0110_0000, // UTF16 String (length 0)
            0b0110_1110, // UTF16 String (length 15)
            0b0110_1111, // UTF16 String (extended payload)
            0b1000_0000, // UID (length 1)
            0b1000_1111, // UID (length 16)
            0b1010_0000, // Array (length 0)
            0b1010_1110, // Array (length 15)
            0b1010_1111, // Array (extended payload)
            0b1101_0000, // Dictionary (length 0)
            0b1101_1110, // Dictionary (length 15)
            0b1101_1111, // Dictionary (extended payload)
        ];
        let expected_results = &[
            (ObjectFormat::Boolean, 0b0000),
            (ObjectFormat::Boolean, 0b0001),
            (ObjectFormat::Fill, 0),
            (ObjectFormat::UInt8, 0),
            (ObjectFormat::UInt16, 0),
            (ObjectFormat::UInt32, 0),
            (ObjectFormat::SInt64, 0),
            (ObjectFormat::Float32, 0),
            (ObjectFormat::Float64, 0),
            (ObjectFormat::Date, 0),
            (ObjectFormat::Data, 0b0000),
            (ObjectFormat::Data, 0b1110),
            (ObjectFormat::Data, 0b1111),
            (ObjectFormat::AsciiString, 0b0000),
            (ObjectFormat::AsciiString, 0b1110),
            (ObjectFormat::AsciiString, 0b1111),
            (ObjectFormat::Utf16String, 0b0000),
            (ObjectFormat::Utf16String, 0b1110),
            (ObjectFormat::Utf16String, 0b1111),
            (ObjectFormat::Uid, 0b0000),
            (ObjectFormat::Uid, 0b1111),
            (ObjectFormat::Array, 0b0000),
            (ObjectFormat::Array, 0b1110),
            (ObjectFormat::Array, 0b1111),
            (ObjectFormat::Dictionary, 0b0000),
            (ObjectFormat::Dictionary, 0b1110),
            (ObjectFormat::Dictionary, 0b1111),
        ];
        for i in 0 .. test_markers.len() {
            assert_eq!(
                any_marker(&test_markers[i .. ]),
                Ok((&test_markers[i+1 .. ], expected_results[i])),
            );
        }
    }

    #[test]
    fn test_boolean() {
        let test_input = &[
            // Boolean(false)
            0b0000_1000,
            // Boolean(true)
            0b0000_1001,
        ];
        let expected_output = vec![
            false,
            true,
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, boolean)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_fill() {
        let test_input = &[
            // Fill
            0b0000_1111,
            // Fill
            0b0000_1111,
            // Fill
            0b0000_1111,
        ];
        let expected_output = vec![
            (),
            (),
            (),
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, fill)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_uint8() {
        let test_input = &[
            // UInt8(0)
            0b0001_0000, 0x00,
            // UInt8(5)
            0b0001_0000, 0x05,
            // UInt8(255)
            0b0001_0000, 0xFF,
        ];
        let expected_output = vec![
            0,
            5,
            255
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, uint8)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_uint16() {
        let test_input = &[
            // UInt16(0)
            0b0001_0001, 0x00, 0x00,
            // UInt16(85)
            0b0001_0001, 0x00, 0x55,
            // UInt16(65535)
            0b0001_0001, 0xFF, 0xFF,
        ];
        let expected_output = vec![
            0,
            85,
            65535,
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, uint16)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_uint32() {
        let test_input = &[
            // UInt32(0)
            0b0001_0010, 0x00, 0x00, 0x00, 0x00,
            // UInt32(21845)
            0b0001_0010, 0x00, 0x00, 0x55, 0x55,
            // UInt32(MAX)
            0b0001_0010, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let expected_output = vec![
            0,
            21845,
            4294967295,
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, uint32)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_sint64() {
        let test_input = &[
            // SInt64(0)
            0b0001_0011, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // SInt64(21845)
            0b0001_0011, 0x00, 0x00, 0x00, 0x00, 0x55, 0x55, 0x55, 0x55,
            // SInt64(-1)
            0b0001_0011, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let expected_output = vec![
            0,
            1431655765,
            -1,
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, sint64)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_float32() {
        let test_input = &[
            // Float32(0)
            0b0010_0010, 0x00, 0x00, 0x00, 0x00,
            // Float32(-2.5)
            0b0010_0010, 0xC0, 0x20, 0x00, 0x00,
            // Float32(40.1328125)
            0b0010_0010, 0x42, 0x20, 0x88, 0x00,
        ];
        let expected_output = vec![
            0.0,
            -2.5,
            40.1328125,
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, float32)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_float64() {
        let test_input = &[
            // Float64(0)
            0b0010_0011, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Float64(-2.5)
            0b0010_0011, 0xC0, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Float64(40.1328125)
            0b0010_0011, 0x40, 0x44, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let expected_output = vec![
            0.0,
            -2.5,
            40.1328125,
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, float64)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_date() {
        let test_input = &[
            // Date(CFAbsoluteTime = 0)
            0b0011_0011, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Date(CFAbsoluteTime = -2.5)
            0b0011_0011, 0xC0, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Date(CFAbsoluteTime = 40.1328125)
            0b0011_0011, 0x40, 0x44, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let expected_output = vec![
            0.0,
            -2.5,
            40.1328125,
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, date)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_data() {
        let test_input = &[
            // Data([length = 0, encoded])
            0b0100_0000,
            // Data([length = 1, encoded])
            0b0100_0001, 0x0F,
            // Data([length = 14, encoded])
            0b0100_1110, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x06, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            // Data([length = 0, trailing])
            0b0100_1111, 0b0001_0000, 0b0000_0000,
            // Data([length = 1, trailing])
            0b0100_1111, 0b0001_0000, 0b0000_0001, 0x0F,
        ];
        let expected_output = vec![
            &test_input[1 .. 1],
            &test_input[2 .. 3],
            &test_input[4 .. 18],
            &test_input[21 .. 21],
            &test_input[24 .. 25],
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, data)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_ascii_string() {
        let test_input = &[
            // AsciiString("", encoded)
            0b0101_0000,
            // AsciiString("Hello", encoded)
            0b0101_0101, 0x48, 0x65, 0x6c, 0x6c, 0x6f,
            // AsciiString("", trailing)
            0b0101_1111, 0b0001_0000, 0b0000_0000,
            // AsciiString("Hello", trailing)
            0b0101_1111, 0b0001_0000, 0b0000_0101, 0x48, 0x65, 0x6c, 0x6c, 0x6f,
        ];
        let expected_output = vec![
            "",
            "Hello",
            "",
            "Hello",
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, ascii_string)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_ascii_string_invalid() {
        // Invalid ASCII string with an 8-bit value.
        let test_input = &[
            0b0101_0001, 0x80,
        ];
        assert_eq!(ascii_string(test_input).is_err(), true);
    }

    #[test]
    fn test_utf16_string() {
        let test_input = &[
            // Utf16String("", encoded)
            0b0110_0000,
            // Utf16String("Hello", encoded)
            0b0110_0101, 0x00, 0x48, 0x00, 0x65, 0x00, 0x6c, 0x00, 0x6c, 0x00, 0x6f,
            // Utf16String("", trailing)
            0b0110_1111, 0b0001_0000, 0b0000_0000,
            // Utf16String("Hello", trailing)
            0b0110_1111, 0b0001_0000, 0b0000_0101, 0x00, 0x48, 0x00, 0x65, 0x00, 0x6c, 0x00, 0x6c, 0x00, 0x6f,
        ];
        let expected_output = vec![
            String::from(""),
            String::from("Hello"),
            String::from(""),
            String::from("Hello"),
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, utf16_string)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_uid() {
        let test_input = &[
            // Uid([length = 1])
            0b1000_0000, 0x00,
            // Uid([length = 16, encoded])
            0b1000_1111, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x06, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        ];
        let expected_output = vec![
            &test_input[1 .. 2],
            &test_input[3 .. 19],
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, uid)(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_array() {
        let test_input = &[
            // Array(reference_size = 2, length = 0, encoded)
            0b1010_0000,
            // Array(reference_size = 2, length = 4, encoded)
            0b1010_0100, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03,
            // Array(reference_size = 2, length = 0, trailing: uint8)
            0b1010_1111, 0b0001_0000, 0b0000_0000,
            // Array(reference_size = 2, length = 3, trailing: uint8)
            0b1010_1111, 0b0001_0000, 0b0000_0011, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02,
        ];
        let expected_output = vec![
            vec![],
            vec![0, 1, 2, 3],
            vec![],
            vec![0, 1, 2],
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, array(2))(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }

    #[test]
    fn test_dictionary() {
        let test_input = &[
            // Dictionary(reference_size = 2, length = 0, encoded)
            0b1101_0000,
            // Dictionary(reference_size = 2, length = 2, encoded)
            0b1101_0010, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03,
            // Dictionary(reference_size = 2, length = 0, trailing: uint8)
            0b1101_1111, 0b0001_0000, 0b0000_0000,
            // Dictionary(reference_size = 2, length = 2, trailing: uint8)
            0b1101_1111, 0b0001_0000, 0b0000_0010, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03,
        ];
        let expected_output = vec![
            vec![],
            vec![(0, 2), (1, 3)],
            vec![],
            vec![(0, 2), (1, 3)],
        ];
        let count = expected_output.len();
        assert_eq!(
            many_m_n(count, count, dictionary(2))(test_input),
            Ok((
                &test_input[test_input.len() .. ],
                expected_output,
            ))
        );
    }
}
