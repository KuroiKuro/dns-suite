use nom::{bits::complete::take, IResult, number};

/// Parse `count` bits from the input. The input should be a tuple containing the
/// input byte slice, and the offset of the slice to parse from. The returned value
/// is a tuple containing a tuple of the remaining input and the current offset, and
/// the second value in the tuple is the parsed bit value as a `u8`
pub fn bit_parser(input: (&[u8], usize), count: usize) -> IResult<(&[u8], usize), u8> {
    take(count)(input)
}

pub fn byte_parser(input: &[u8], count: usize) -> IResult<&[u8], &[u8]> {
    nom::bytes::complete::take(count)(input)
}

/// General function for parsing a `u16` from a sequence of bytes
pub fn parse_u16(bytes: &[u8]) -> IResult<&[u8], u16> {
    let (remaining_input, parsed) = byte_parser(bytes, 2)?;
    let (_, parsed_u16) = number::complete::be_u16(parsed)?;
    Ok((remaining_input, parsed_u16))
}

/// General function for parsing a `i32` from a sequence of bytes
pub fn parse_i32(bytes: &[u8]) -> IResult<&[u8], i32> {
    let (remaining_input, parsed) = byte_parser(bytes, 4)?;
    let (_, parsed_i32) = number::complete::be_i32(parsed)?;
    Ok((remaining_input, parsed_i32))
}

/// General function for parsing a `u32` from a sequence of bytes
pub fn parse_u32(bytes: &[u8]) -> IResult<&[u8], u32> {
    let (remaining_input, parsed) = byte_parser(bytes, 4)?;
    let (_, parsed_u32) = number::complete::be_u32(parsed)?;
    Ok((remaining_input, parsed_u32))
}
