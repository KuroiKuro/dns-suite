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
    let first_result = parser((inp, 0usize), 8);
    // let result = take::<&[u8], &[u8], usize, E>(2usize)((inp, 0usize));
    let ((modified_input, something), first_byte) = first_result.unwrap();
    println!("First byte:");
    println!("modified_input = {:?}", modified_input);
    println!("Something = {}", something);
    println!("output = 0b{:b}", first_byte);
    let second_result = parser((modified_input, 0usize), 8);
    let ((modified_input, read_bits), second_byte) = second_result.unwrap();
    println!("Second byte:");
    println!("modified_input = {:?}", modified_input);
    println!("Something = {}", read_bits);
    println!("output = 0b{:b}", second_byte);
}

pub fn test() {
    parse_id(&TEST_ARR);
}
