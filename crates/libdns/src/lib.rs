use idna::punycode;
use itertools::{Itertools, Position};
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

impl From<&str> for DomainLabel {
    /// TODO: DNS actually uses ASCII, unless using the IDNA specification specified
    /// in RFC 5890. Also change this to `impl TryFrom` to return `Result`
    fn from(value: &str) -> Self {
        let len = value.len();
        let punycode_str = punycode::decode_to_string(value).unwrap();
        if !Self::validate_label(&punycode_str) {
            panic!("Invalid label!")
        }
        let str_bytes = value.as_bytes();
        let byte_repr = match len {
            0 => vec![0],
            _ => [&[len as u8], str_bytes].concat()
        };
        Self { len, byte_repr }
    }
}

impl DomainLabel {
    fn validate_label(label: &str) -> bool {
        let mut chars = label.clone().chars();
        let label_len = label.len();
        if label_len > 63 {
            return false;
        }
        let validated_chars: Vec<char> = chars.with_position()
            .map_while(|(pos, c)| {
                if
                    (pos == Position::First && !c.is_alphabetic()) ||
                    (!c.is_alphanumeric() && c != '-') ||
                    (pos == Position::Last && !c.is_alphanumeric()) {
                    None
                } else {
                    Some(c)
                }
            })
            .collect();

        validated_chars.len() == label_len
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

pub struct DomainName {
    domain_labels: Vec<DomainLabel>,
    domain_name: String,
}

impl TryFrom<&str> for DomainName {
    // TODO: Create proper Error type!
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let split = value.split(".");
        // if split.clone().count() == 0 {
        //     return Self::Error;
        // }
        split.map(|domain_part| {

        })
    }
}

// Placeholders
struct Header;
struct Question;
struct Answer;
struct Authority;
struct Additional;

pub struct DnsQuery {
    header: Header,
    question: Question,
    answer: Answer,
    authority: Authority,
    additional: Additional,
}

pub struct DnsAnswer {

}
