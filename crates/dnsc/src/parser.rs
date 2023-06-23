use nom::{bits::complete::take, IResult};

const TEST_ARR: [u8; 4] = [
    0b1111_1101,
    0b0000_0010,
    0b1111_0000,
    0b1111_0000,
];

/*
    MESSAGE HEADER PARSING
*/


/// Parse `count` bits from the input. The input should be a tuple containing the
/// input byte slice, and the offset of the slice to parse from
fn bit_parser(input: (&[u8], usize), count: usize)-> IResult<(&[u8], usize), u8> {
    take(count)(input)
}

fn byte_parser(input: &[u8], count: usize) -> IResult<&[u8], &[u8]> {
    nom::bytes::complete::take(count)(input)
}


/// Parse the query ID from the DNS message's header
fn parse_id(inp: &[u8]) -> IResult<&[u8], u16> {
    // Take the first 2 bytes, and get a u16 from it
    let take_bytes_result = byte_parser(inp, 2);
    let (new_input, take_bytes) = take_bytes_result.unwrap();
    let parsed_id_result = nom::number::complete::be_u16::<&[u8], nom::error::Error<&[u8]>>(take_bytes);
    let (_, parsed_id) = parsed_id_result.unwrap();
    Ok((new_input, parsed_id))
}

pub fn test() {
    let (new_input, parsed_id) = parse_id(&TEST_ARR).unwrap();
    println!("parsed_id = {:b}", parsed_id);
    dbg!(new_input);
}
