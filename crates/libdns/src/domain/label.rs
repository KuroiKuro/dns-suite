// use idna::punycode;

use std::cmp::PartialEq;
use std::str::FromStr;

use ascii::{AsciiChar, AsciiStr, AsciiString};
use itertools::{Itertools, Position};
use thiserror::Error;

const MAX_LABEL_LENGTH: usize = 63;
// TODO: enable punycode in future
// const ENABLE_PUNYCODE: bool = false;

#[derive(Error, Debug)]
pub enum DomainLabelValidationError {
    #[error(
        "Domain label ({0}) was {1} chars long, exceeding max length of {}",
        MAX_LABEL_LENGTH
    )]
    LabelTooLong(String, usize),
    #[error("Invalid starting character '{1}' in domain label '{0}'")]
    InvalidStartChar(String, AsciiChar),
    #[error("Invalid ending character '{1}' in domain label '{0}'")]
    InvalidEndChar(String, AsciiChar),
    #[error("Invalid character '{1}' in domain label '{0}'")]
    InvalidChar(String, AsciiChar),
    #[error("Unable to parse ASCII characters from domain label '{0}'")]
    InvalidAscii(String),
}

/// Represents a label within a domain name. According to RFC 1035 Section 3.1,
/// "Domain names in messages are expressed in terms of a sequence of labels.
/// Each label is represented as a one octet length field followed by that
/// number of octets.  Since every domain name ends with the null label of
/// the root, a domain name is terminated by a length byte of zero."
///
/// Note that in the current implementation, IDNA is not supported, and only
/// pure ASCII characters for domain labels are supported
#[derive(Debug)]
pub struct DomainLabel {
    len: usize,
    byte_repr: Vec<u8>,
    label_str: AsciiString,
}

impl TryFrom<&str> for DomainLabel {
    type Error = DomainLabelValidationError;
    /// TODO: DNS actually uses ASCII, unless using the IDNA specification specified
    /// in RFC 5890.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let ascii_value = match AsciiString::from_str(value) {
            Ok(val) => val,
            Err(_) => return Err(DomainLabelValidationError::InvalidAscii(value.to_string())),
        };
        let len = ascii_value.len();
        Self::validate_label(&ascii_value)?;
        let str_bytes = value.as_bytes();
        let byte_repr = match len {
            0 => vec![0],
            _ => [&[len as u8], str_bytes].concat(),
        };
        Ok(Self {
            len,
            byte_repr,
            label_str: ascii_value,
        })
    }
}

impl PartialEq for DomainLabel {
    fn eq(&self, other: &Self) -> bool {
        // Labels are case insensitive for comparison purposes in the DNS spec
        let self_label = self.label_str.to_ascii_lowercase();
        let other_label = other.label_str.to_ascii_lowercase();
        self_label == other_label
    }
}

impl DomainLabel {
    fn validate_label(label: &AsciiStr) -> Result<(), DomainLabelValidationError> {
        let chars = label.clone().chars();
        let label_len = label.len();
        if label_len > MAX_LABEL_LENGTH {
            return Err(DomainLabelValidationError::LabelTooLong(
                label.to_string(),
                label_len,
            ));
        }

        for (pos, ch) in chars.with_position() {
            if pos == Position::First && !ch.is_alphabetic() {
                return Err(DomainLabelValidationError::InvalidStartChar(
                    label.to_string(),
                    ch,
                ));
            } else if pos == Position::Last && !ch.is_alphanumeric() {
                return Err(DomainLabelValidationError::InvalidEndChar(
                    label.to_string(),
                    ch,
                ));
            } else if ch != AsciiChar::Minus && !ch.is_alphanumeric() {
                return Err(DomainLabelValidationError::InvalidChar(
                    label.to_string(),
                    ch,
                ));
            }
        }
        Ok(())
    }

    /// Creates a new empty `DomainLabel` instance. Mainly for use of terminating
    /// domain names, which are terminanted with a null label
    pub fn new_empty() -> Self {
        Self {
            len: 0,
            byte_repr: vec![0],
            label_str: AsciiString::new(),
        }
    }

    /// Returns a bytes slice representing the domain label. Following the spec, the
    /// first element of the slice will be the length of the label, followed by the
    /// bytes of the label itself
    pub fn as_bytes(&self) -> &[u8] {
        &self.byte_repr
    }

    /// Returns the length of the label, not the total length of the byte slice
    /// that will be returned by `as_bytes`
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_instantiation() -> Result<(), DomainLabelValidationError> {
        let valid_label1 = "com";
        let valid_label2 = "google";
        DomainLabel::try_from(valid_label1)?;
        DomainLabel::try_from(valid_label2)?;
        Ok(())
    }

    #[test]
    fn test_byte_repr() {
        // Test for label: "com". The label representation should be the len
        // of the label + the bytes
        let test_vec: Vec<u8> = vec![3, 99, 111, 109];
        let label = DomainLabel::try_from("com").unwrap();
        assert_eq!(test_vec, label.byte_repr);
    }

    #[test]
    fn test_eq() {
        let label1 = DomainLabel::try_from("com").unwrap();
        let label2 = DomainLabel::try_from("CoM").unwrap();
        assert_eq!(label1, label2);
    }
}
