use ascii::AsciiString;
use thiserror::Error;

pub const MAX_CHARACTER_STRING_LEN: usize = 256;

#[derive(Debug, Error)]
pub enum CharacterStringError {
    #[error("String '{0}' is too long for character string")]
    TooLong(AsciiString, usize),
}

/// A struct representing a `<character-string>`, defined in section 3.3 of RDC 1035.
/// <character-string> is a single length octet followed by that number of characters.
/// <character-string> is treated as binary information, and can be up to 256 characters
/// in length (including the length octet).
#[derive(Debug)]
pub struct CharacterString {
    /// The length of the character string
    len: usize,
    /// The original ASCII representation of the character string
    char_str: AsciiString,
    bytes_repr: Vec<u8>,
}

impl TryFrom<AsciiString> for CharacterString {
    type Error = CharacterStringError;

    fn try_from(value: AsciiString) -> Result<Self, Self::Error> {
        let len = value.len();
        // Add 1 to include the value of the string's length
        if len + 1 > MAX_CHARACTER_STRING_LEN {
            return Err(CharacterStringError::TooLong(value, MAX_CHARACTER_STRING_LEN));
        }
        let bytes_repr = Self::ascii_to_bytes(&value, len);
        Ok(Self { len: value.len(), char_str: value, bytes_repr })
    }
}

impl CharacterString {
    /// Encodes the data of the current `CharacterString` into a new `Vec<u8>`
    fn ascii_to_bytes(char_str: &AsciiString, len: usize) -> Vec<u8> {
        let mut bytes_repr: Vec<u8> = vec![len as u8];
        bytes_repr.extend(char_str.as_bytes());
        bytes_repr
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn char_str(&self) -> &str {
        self.char_str.as_ref()
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn byte_slice(&self) -> &[u8] {
        self.bytes_repr.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    
    #[test]
    fn test_validation() {
        let too_long_str = (0..257).map(|_| 's')
            .collect::<String>();
        assert!(CharacterString::try_from(AsciiString::from_str(&too_long_str).unwrap()).is_err());
    }

    #[test]
    fn test_bytes_repr() {
        let char_str1 = CharacterString::try_from(AsciiString::from_str("Abcde").unwrap()).unwrap();
        let expected_bytes1: Vec<u8> = vec![5, 65, 98, 99, 100, 101];
        assert_eq!(char_str1.byte_slice(), &expected_bytes1);
        let empty_char_str = CharacterString::try_from(AsciiString::new()).unwrap();
        let expected_bytes2: Vec<u8> = vec![0];
        assert_eq!(empty_char_str.byte_slice(), &expected_bytes2);
    }
}
