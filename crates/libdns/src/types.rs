use ascii::AsciiString;
use thiserror::Error;

use crate::{parse_utils::{byte_parser, bit_parser}, BytesSerializable, ParseDataError};

pub const MAX_CHARACTER_STRING_LEN: usize = 256;

#[derive(Debug, Error)]
pub enum CharacterStringError {
    #[error("String '{0}' is too long for character string")]
    TooLong(AsciiString, usize),
    #[error("Invalid byte structure")]
    InvalidByteStructure,
}

/// A struct representing a `<character-string>`, defined in section 3.3 of RDC 1035.
/// <character-string> is a single length octet followed by that number of characters.
/// <character-string> is treated as binary information, and can be up to 256 characters
/// in length (including the length octet).
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct CharacterString {
    /// The length of the character string
    len: u8,
    /// The original ASCII representation of the character string
    char_str: AsciiString,
}

impl TryFrom<AsciiString> for CharacterString {
    type Error = CharacterStringError;

    fn try_from(value: AsciiString) -> Result<Self, Self::Error> {
        let len = value.len();
        // Add 1 to include the value of the string's length
        if len + 1 > MAX_CHARACTER_STRING_LEN {
            return Err(CharacterStringError::TooLong(
                value,
                MAX_CHARACTER_STRING_LEN,
            ));
        }
        Ok(Self {
            len: len as u8,
            char_str: value,
        })
    }
}

impl CharacterString {
    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn char_str(&self) -> &str {
        self.char_str.as_ref()
    }

    pub fn ascii_str(&self) -> &AsciiString {
        &self.char_str
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.char_str.as_bytes()
    }
}

impl BytesSerializable for CharacterString {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_repr: Vec<u8> = vec![self.len];
        bytes_repr.extend(self.char_str.as_bytes());
        bytes_repr
    }

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), ParseDataError> {
        let (remaining_input, parsed) =
            byte_parser(bytes, 1).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let len = parsed[0];
        let (remaining_input, parsed) = byte_parser(remaining_input, len as usize)
            .map_err(|_| ParseDataError::InvalidByteStructure)?;
        let char_str =
            AsciiString::from_ascii(parsed).map_err(|_| ParseDataError::InvalidByteStructure)?;
        Ok((
            Self {
                len: char_str.len() as u8,
                char_str,
            },
            remaining_input,
        ))
    }
}

/// A data type representing a pointer to 1 or more domain labels, included in an
/// earlier section of a DNS message. This is to support the DNS message compression
/// specification in RFC 1035, Section 4.1.4
pub struct DomainPointer {
    offset: u16,
}

impl DomainPointer {
    const OFFSET_INDICATOR: u16 = 0xC000;
    const OFFSET_INDICATOR_BITS: u8 = 0b11;

    pub fn new(offset: u16) -> Self {
        Self { offset }
    }
}

impl BytesSerializable for DomainPointer {
    fn to_bytes(&self) -> Vec<u8> {
        // Based on the spec, a domain pointer will start with two `1` bits
        // Since a domain pointer is always two octets (16 bits), and we always
        // need to use `11` as the starting bits, we have no choice but to "discard"
        // the first 2 bits of an offset
        let data = Self::OFFSET_INDICATOR | self.offset;
        data.to_be_bytes().to_vec()
    }

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), ParseDataError> {
        // let first_byte = bytes.first().unwrap();
        let (remaining_input, parsed) = bit_parser((bytes, 0), 2).map_err(|_| ParseDataError::InvalidByteStructure)?;
        if parsed != Self::OFFSET_INDICATOR_BITS {
            return Err(ParseDataError::InvalidByteStructure);
        }
        let (remaining_input, remaining_first_byte) = bit_parser(remaining_input, 6).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (remaining_input, second_byte) = byte_parser(remaining_input.0, 1).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let offset = (remaining_first_byte as u16) << 8 | second_byte[0] as u16;
        Ok((Self::new(offset), remaining_input))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ascii::AsciiChar;

    use super::*;

    #[test]
    fn test_validation() {
        let too_long_str = (0..257).map(|_| 's').collect::<String>();
        assert!(CharacterString::try_from(AsciiString::from_str(&too_long_str).unwrap()).is_err());
    }

    #[test]
    fn test_bytes_repr() {
        let char_str1 = CharacterString::try_from(AsciiString::from_str("Abcde").unwrap()).unwrap();
        let expected_bytes1: Vec<u8> = vec![5, 65, 98, 99, 100, 101];
        assert_eq!(char_str1.to_bytes(), expected_bytes1);
        let empty_char_str = CharacterString::try_from(AsciiString::new()).unwrap();
        let expected_bytes2: Vec<u8> = vec![0];
        assert_eq!(empty_char_str.to_bytes(), expected_bytes2);
    }

    #[test]
    fn test_character_string_parse() {
        let bytes = [
            6,
            AsciiChar::y as u8,
            AsciiChar::e as u8,
            AsciiChar::l as u8,
            AsciiChar::l as u8,
            AsciiChar::o as u8,
            AsciiChar::w as u8,
        ];

        let ascii_s = AsciiString::from_str("yellow").unwrap();
        let expected_label = CharacterString::try_from(ascii_s).unwrap();
        let (domain_label, remaining) = CharacterString::parse(&bytes).unwrap();
        assert_eq!(domain_label, expected_label);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_parse_domain_pointer() {
        let domain_ptr_bytes: [u8; 2] = [
            0b1100_0000,
            0b0000_0111
        ];
        let (domain_ptr, remaining_input) = DomainPointer::parse(&domain_ptr_bytes).unwrap();
        assert_eq!(domain_ptr.offset, 7);
        assert_eq!(remaining_input.len(), 0);

        let domain_ptr_bytes: [u8; 2] = [
            0b1100_1110,
            0b1110_1011
        ];
        let (domain_ptr, remaining_input) = DomainPointer::parse(&domain_ptr_bytes).unwrap();
        assert_eq!(domain_ptr.offset, 3819);
        assert_eq!(remaining_input.len(), 0);
    }
}
