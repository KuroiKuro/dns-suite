use nom::{bits::complete::take, IResult};

const TEST_INPUT: u32 = 0b1111_1111_0000_0000_1111_0000_1111_0000;
const TEST_ARR: [u8; 4] = [
    0b1111_1111,
    0b0000_0000,
    0b1111_0000,
    0b1111_0000,
];

/*
    MESSAGE HEADER PARSING
*/

fn parser(input: (&[u8], usize), count: usize)-> IResult<(&[u8], usize), u8> {
    take(count)(input)
}

// fn parse_id(inp: &[u8]) -> IResult<&[u8], u16> {
fn parse_id(inp: &[u8]) {
    // assert_eq!(
    //     parser(([0b00010010].as_ref(), 0), 4),
    //     Ok((([0b00010010].as_ref(), 4), 0b00000001))
    // );
    
    // Take the first 2 bytes, and get a u16 from it
    let result = parser((inp, 0usize), 8);
    // let result = take::<&[u8], &[u8], usize, E>(2usize)((inp, 0usize));
    let ((modified_input, something), output) = result.unwrap();
    println!("modified_input = {:?}", modified_input);
    println!("Something = {}", something);
    println!("output = {:b}", output);
}

pub fn test() {
    parse_id(&TEST_ARR);
}
