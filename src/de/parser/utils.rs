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
    combinator::map_res,
    bytes::complete::take,
};

use std::convert::TryFrom;

/// Returns a parser which recognizes a variable-length big-endian number
/// between 1 and 8 bytes long, inclusive.
///
/// # Notes
///
/// 1. A valid result will be returned for unsigned values between 1 and 8 bytes.
/// 2. A valid result will be returned for a signed value of exactly 8 bytes.
pub fn be_u64_n(
    n: usize
) -> impl Fn(&[u8]) -> IResult<&[u8], u64> {
    assert!(n >= 1 && n <= 8, "number must be between 1 and 8 bytes, inclusive");
    move |input: &[u8]| {
        let (input, bytes) = take(n)(input)?;
        let value = bytes.iter().fold(0u64, |acc, x| {
            (acc << 8) + *x as u64
        });
        Ok((input, value))
    }
}

/// Returns a parser which recognizes a variable-length unsigned big-endian number
/// between 1 and 8 bytes long, inclusive. This value is then converted safely
/// into a usize, which varies based on the pointer size of the platform.
///
/// # Notes
///
/// 1. n may be up to 8 even on platforms with smaller word sizes.
/// 2. The value is checked to confirm it is in range before it is cast to usize.
pub fn be_usize_n(
    n: usize
) -> impl Fn(&[u8]) -> IResult<&[u8], usize> {
    move |input: &[u8]| {
        map_res(
            be_u64_n(n),
            |value| usize::try_from(value)
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::{be_u64_n, be_usize_n};

    #[test]
    fn test_be_usize_n_length_1() {
        let (input, result) = be_usize_n(1)(&[0x05]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_be_usize_n_length_3() {
        let (input, result) = be_usize_n(3)(&[0x00, 0x01, 0xFF]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(result, 511);
    }

    #[test]
    fn test_be_u64_n_length_8() {
        let (input, result) = be_u64_n(8)(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xFF]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(result, 72057594037928447);
    }
}
