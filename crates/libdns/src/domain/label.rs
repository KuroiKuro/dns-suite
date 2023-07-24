use idna::punycode;
use itertools::{Itertools, Position};
use thiserror::Error;

const MAX_LABEL_LENGTH: usize = 63;

#[derive(Error, Debug)]
pub enum DomainLabelValidationError {
    #[error("Domain label ({0}) was {1} chars long, exceeding max length of {}", MAX_LABEL_LENGTH)]
    LabelTooLong(String, usize),
    #[error("Invalid starting character '{1}' in domain label '{0}'")]
    InvalidStartChar(String, char),
    #[error("Invalid ending character '{1}' in domain label '{0}'")]
    InvalidEndChar(String, char),
    #[error("Invalid character '{1}' in domain label '{0}'")]
    InvalidChar(String, char),
}

/// Represents a label within a domain name. According to RFC 1035 Section 3.1,
/// "Domain names in messages are expressed in terms of a sequence of labels.
/// Each label is represented as a one octet length field followed by that
/// number of octets.  Since every domain name ends with the null label of
/// the root, a domain name is terminated by a length byte of zero."
pub struct DomainLabel {
    len: usize,
    byte_repr: Vec<u8>,
}

impl From<&[u8]> for DomainLabel {
    fn from(value: &[u8]) -> Self {
        let len = value.len();
        let byte_repr = match len {
            0 => vec![0],
            _ => [&[len as u8], value].concat()
        };
        Self { len, byte_repr }
    }
}

impl TryFrom<&str> for DomainLabel {
    type Error = DomainLabelValidationError;
    /// TODO: DNS actually uses ASCII, unless using the IDNA specification specified
    /// in RFC 5890. Also change this to `impl TryFrom` to return `Result`
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let len = value.len();
        let punycode_str = punycode::decode_to_string(value).unwrap();
        if let Err(e) = Self::validate_label(&punycode_str) {
            return Err(e);
        }
        let str_bytes = value.as_bytes();
        let byte_repr = match len {
            0 => vec![0],
            _ => [&[len as u8], str_bytes].concat()
        };
        Ok(Self { len, byte_repr })
    }
}

impl DomainLabel {
    fn validate_label(label: &str) -> Result<(), DomainLabelValidationError> {
        let mut chars = label.clone().chars();
        let label_len = label.len();
        if label_len > MAX_LABEL_LENGTH {
            return Err(DomainLabelValidationError::LabelTooLong(label.to_string(), label_len));
        }

        for (pos, c) in chars.with_position() {
            if pos == Position::First && !c.is_alphabetic() {
                return Err(DomainLabelValidationError::InvalidStartChar(label.to_string(), c));
            }
            else if pos == Position::Last && !c.is_alphanumeric() {
                return Err(DomainLabelValidationError::InvalidEndChar(label.to_string(), c));
            }
            else if c != '-' || !c.is_alphanumeric() {
                return Err(DomainLabelValidationError::InvalidChar(label.to_string(), c));
            }
        }
        Ok(())
    }

    /// Creates a new empty `DomainLabel` instance. Mainly for use of terminating
    /// domain names, which are terminanted with a null label
    pub fn new_empty() -> Self {
        Self { len: 0, byte_repr: vec![0] }
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