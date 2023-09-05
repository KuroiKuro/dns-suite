use nom::{bits::complete::take, IResult};

/// Parse `count` bits from the input. The input should be a tuple containing the
/// input byte slice, and the offset of the slice to parse from
pub fn bit_parser(input: (&[u8], usize), count: usize) -> IResult<(&[u8], usize), u8> {
    take(count)(input)
}

pub fn byte_parser(input: &[u8], count: usize) -> IResult<&[u8], &[u8]> {
    nom::bytes::complete::take(count)(input)
}
