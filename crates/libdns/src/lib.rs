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
    fn from(value: &str) -> Self {
        let len = value.len();
        let str_bytes = value.as_bytes();
        let byte_repr = match len {
            0 => vec![0],
            _ => [&[len as u8], str_bytes].concat()
        };
        Self { len, byte_repr }
    }
}

impl DomainLabel {
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
