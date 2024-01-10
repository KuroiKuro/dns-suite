use crate::{
    parse_utils::{bit_parser, parse_u16},
    BytesSerializable, ParseDataError,
};

use super::{MessageType, QueryOpcode, ResponseCode};
use itertools::Itertools;
use nom::IResult;
use rand::random;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseHeaderError {
    #[error("Error parsing ID in message header")]
    IdError,
    #[error("Error parsing Message Type (QR) in message header")]
    QrError,
    #[error("Error parsing Opcode in message header")]
    OpcodeError,
    #[error("Error parsing Authoritative Answer (AA) in message header")]
    AaError,
    #[error("Error parsing Truncation (TC) in message header")]
    TcError,
    #[error("Error parsing Recursion Desired (RD) in message header")]
    RdError,
    #[error("Error parsing Recursion Available (RA) in message header")]
    RaError,
    #[error("Error parsing Response Code (Rcode) in message header")]
    RcodeError,
    #[error("Error parsing QDCOUNT in message header")]
    QdcountError,
    #[error("Error parsing ANCOUNT in message header")]
    AncountError,
    #[error("Error parsing NSCOUNT in message header")]
    NscountError,
    #[error("Error parsing ARCOUNT in message header")]
    ArcountError,
}

/// A DNS message header. The header contains the following fields:
///                               1  1  1  1  1  1
/// 0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                      ID                       |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    QDCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    ANCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    NSCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    ARCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
struct Header {
    /// ID: A 16 bit identifier assigned by the program that generates any kind of query.
    /// This identifier is copied the corresponding reply and can be used by the requester
    /// to match up replies to outstanding queries.
    id: u16,
    /// QR: A one bit field that specifies whether this message is a query (0), or a response (1)
    qr: MessageType,
    /// OPCODE: A four bit field that specifies kind of query in this message. This value is set
    /// by the originator of a query and copied into the response.
    opcode: QueryOpcode,
    /// AA: this bit is valid in responses, and specifies that the responding name server is an
    /// authority for the domain name in question section. Note that the contents of the answer
    /// section may have multiple owner names because of aliases. The AA bit corresponds to the
    /// name which matches the query name, or the first owner name in the answer section.
    authoritative_ans: bool,
    /// TC: specifies that this message was truncated due to length greater than that permitted
    /// on the transmission channel.
    truncation: bool,
    /// RD: this bit may be set in a query and is copied into the response. If RD is set, it
    /// directs the name server to pursue the query recursively. Recursive query support is optional.
    recursion_desired: bool,
    /// RA: this bit is set or cleared in a response, and denotes whether recursive query support is
    /// available in the name server
    recursion_available: bool,
    /// RCODE: this 4 bit field is set as part of responses.
    response_code: ResponseCode,
    /// an unsigned 16 bit integer specifying the number of entries in the question section.
    qdcount: u16,
    /// an unsigned 16 bit integer specifying the number of resource records in the answer section.
    ancount: u16,
    /// an unsigned 16 bit integer specifying the number of name server resource records in the
    /// authority records section.
    nscount: u16,
    /// an unsigned 16 bit integer specifying the number of resource records in the additional
    /// records section.
    arcount: u16,
}

impl Header {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u16,
        qr: MessageType,
        opcode: QueryOpcode,
        authoritative_ans: bool,
        truncation: bool,
        recursion_desired: bool,
        recursion_available: bool,
        response_code: ResponseCode,
        qdcount: u16,
        ancount: u16,
        nscount: u16,
        arcount: u16,
    ) -> Self {
        Self {
            id,
            qr,
            opcode,
            authoritative_ans,
            truncation,
            recursion_desired,
            recursion_available,
            response_code,
            qdcount,
            ancount,
            nscount,
            arcount,
        }
    }

    pub fn builder(qr: MessageType) -> HeaderBuilder {
        HeaderBuilder::new(qr)
    }

    fn second_section(&self) -> u16 {
        let qr = (self.qr as u16) << 15;
        let opcode = (self.opcode as u16) << 11;
        let aa = (self.authoritative_ans as u16) << 10;
        let tc = (self.truncation as u16) << 9;
        let rd = (self.recursion_desired as u16) << 8;
        let ra = (self.recursion_available as u16) << 7;
        let z = 0;
        let rcode = self.response_code as u16;
        qr | opcode | aa | tc | rd | ra | z | rcode
    }

    // Parsing functions

    /// Parse the `qr` bit from the given bytes. The returned bit should be casted to
    /// `MessageType` by the caller. As the first bit-level parsing function to be
    /// called, the offset should always be `0`, unless using this parser in a
    /// different context
    fn parse_qr(bytes_with_offset: (&[u8], usize)) -> IResult<(&[u8], usize), u8> {
        bit_parser(bytes_with_offset, 1)
    }

    /// Parse the `opcode` bit from the given bytes. The returned bit should be casted to
    /// `QueryOpcode` by the caller
    fn parse_opcode(bytes_with_offset: (&[u8], usize)) -> IResult<(&[u8], usize), u8> {
        bit_parser(bytes_with_offset, 4)
    }

    fn parse_bool_bit(bytes_with_offset: (&[u8], usize)) -> IResult<(&[u8], usize), bool> {
        let (remaining_input, parsed) = bit_parser(bytes_with_offset, 1)?;
        let parsed_bool = parsed == 1;
        Ok((remaining_input, parsed_bool))
    }

    /// Parse the `rcode` bit from the given bytes. The returned bit should be casted to
    /// `ResponseCode` by the caller
    fn parse_rcode(bytes_with_offset: (&[u8], usize)) -> IResult<(&[u8], usize), u8> {
        // Since rcode is directly after the `Z` section, which is unused in the spec, we will
        // simply use the offset to skip parsing the `Z` section
        let (bytes, offset) = bytes_with_offset;
        let new_offset = offset + 3;
        bit_parser((bytes, new_offset), 4)
    }
}

impl BytesSerializable for Header {
    fn to_bytes(&self) -> Vec<u8> {
        [
            self.id,
            self.second_section(),
            self.qdcount,
            self.ancount,
            self.nscount,
            self.arcount,
        ]
        .iter()
        .flat_map(|val| val.to_be_bytes())
        .collect_vec()
    }

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), ParseDataError> {
        let (bytes, id) = parse_u16(bytes).map_err(|_| ParseDataError::InvalidByteStructure)?;

        let (bytes_with_offset, qr) =
            Self::parse_qr((bytes, 0)).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let qr = MessageType::try_from(qr).map_err(|_| ParseDataError::InvalidByteStructure)?;

        let (bytes_with_offset, opcode) = Self::parse_opcode(bytes_with_offset)
            .map_err(|_| ParseDataError::InvalidByteStructure)?;
        let opcode =
            QueryOpcode::try_from(opcode).map_err(|_| ParseDataError::InvalidByteStructure)?;

        let (bytes_with_offset, aa) = Self::parse_bool_bit(bytes_with_offset)
            .map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (bytes_with_offset, tc) = Self::parse_bool_bit(bytes_with_offset)
            .map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (bytes_with_offset, rd) = Self::parse_bool_bit(bytes_with_offset)
            .map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (bytes_with_offset, ra) = Self::parse_bool_bit(bytes_with_offset)
            .map_err(|_| ParseDataError::InvalidByteStructure)?;

        // The offset shouldn't be used anymore on the last bit parsing action
        let ((bytes, _), rcode) = Self::parse_rcode(bytes_with_offset)
            .map_err(|_| ParseDataError::InvalidByteStructure)?;
        let rcode =
            ResponseCode::try_from(rcode).map_err(|_| ParseDataError::InvalidByteStructure)?;

        let (bytes, qdcount) =
            parse_u16(bytes).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (bytes, ancount) =
            parse_u16(bytes).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (bytes, nscount) =
            parse_u16(bytes).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (bytes, arcount) =
            parse_u16(bytes).map_err(|_| ParseDataError::InvalidByteStructure)?;
        Ok((
            Self {
                id,
                qr,
                opcode,
                authoritative_ans: aa,
                truncation: tc,
                recursion_desired: rd,
                recursion_available: ra,
                response_code: rcode,
                qdcount,
                ancount,
                nscount,
                arcount,
            },
            bytes,
        ))
    }
}

/// A builder type to construct `Header` instances. The only field that is required upfront is the
/// `qr` field. Every other field is optional - see the respective documentation on the field to
/// understand what are the default values that will be used. See the documentation on `Header` to
/// get an overview of what each field represents.
struct HeaderBuilder {
    /// Defaults to generating a random `u16` if not set. This is useful for new DNS queries, which
    /// will use a newly generated ID. Set the ID if it is a response to an existing query
    id: Option<u16>,
    /// The query type that will be set in the header
    qr: MessageType,
    /// Defaults to using `QueryOpcode::Query`
    opcode: QueryOpcode,
    /// Defaults to `false`
    authoritative_ans: bool,
    /// Defaults to `false`
    truncation: bool,
    /// Defaults to `false`
    recursion_desired: bool,
    /// Defaults to `false`
    recursion_available: bool,
    /// Defaults to `ResponseCode::NoError`
    response_code: ResponseCode,
    /// Defaults to `0`
    qdcount: u16,
    /// Defaults to `0`
    ancount: u16,
    /// Defaults to `0`
    nscount: u16,
    /// Defaults to `0`
    arcount: u16,
}

impl HeaderBuilder {
    const DEFAULT_OPCODE: QueryOpcode = QueryOpcode::Query;
    const DEFAULT_AUTHORITATIVE_ANS: bool = false;
    const DEFAULT_TRUNCATION: bool = false;
    const DEFAULT_RECURSION_DESIRED: bool = false;
    const DEFAULT_RECURSION_AVAILABLE: bool = false;
    const DEFAULT_RESPONSE_CODE: ResponseCode = ResponseCode::NoError;
    const DEFAULT_QDCOUNT: u16 = 0;
    const DEFAULT_ANCOUNT: u16 = 0;
    const DEFAULT_NSCOUNT: u16 = 0;
    const DEFAULT_ARCOUNT: u16 = 0;

    fn generate_id() -> u16 {
        random::<u16>()
    }

    pub fn new(qr: MessageType) -> Self {
        Self {
            id: None,
            qr,
            opcode: Self::DEFAULT_OPCODE,
            authoritative_ans: Self::DEFAULT_AUTHORITATIVE_ANS,
            truncation: Self::DEFAULT_TRUNCATION,
            recursion_desired: Self::DEFAULT_RECURSION_DESIRED,
            recursion_available: Self::DEFAULT_RECURSION_AVAILABLE,
            response_code: Self::DEFAULT_RESPONSE_CODE,
            qdcount: Self::DEFAULT_QDCOUNT,
            ancount: Self::DEFAULT_ANCOUNT,
            nscount: Self::DEFAULT_NSCOUNT,
            arcount: Self::DEFAULT_ARCOUNT,
        }
    }

    pub fn finalize(self) -> Header {
        let id = match self.id {
            Some(id) => id,
            None => Self::generate_id(),
        };
        Header {
            id,
            qr: self.qr,
            opcode: self.opcode,
            authoritative_ans: self.authoritative_ans,
            truncation: self.truncation,
            recursion_desired: self.recursion_desired,
            recursion_available: self.recursion_available,
            response_code: self.response_code,
            qdcount: self.qdcount,
            ancount: self.ancount,
            nscount: self.nscount,
            arcount: self.arcount,
        }
    }

    pub fn set_id(mut self, id: u16) -> Self {
        self.id = Some(id);
        self
    }

    pub fn set_opcode(mut self, opcode: QueryOpcode) -> Self {
        self.opcode = opcode;
        self
    }

    pub fn set_authoritative_ans(mut self, authoritative_ans: bool) -> Self {
        self.authoritative_ans = authoritative_ans;
        self
    }

    pub fn set_truncation(mut self, truncation: bool) -> Self {
        self.truncation = truncation;
        self
    }

    pub fn set_recursion_desired(mut self, recursion_desired: bool) -> Self {
        self.recursion_desired = recursion_desired;
        self
    }

    pub fn set_recursion_available(mut self, recursion_available: bool) -> Self {
        self.recursion_available = recursion_available;
        self
    }

    pub fn set_response_code(mut self, response_code: ResponseCode) -> Self {
        self.response_code = response_code;
        self
    }

    pub fn set_qdcount(mut self, qdcount: u16) -> Self {
        self.qdcount = qdcount;
        self
    }

    pub fn set_ancount(mut self, ancount: u16) -> Self {
        self.ancount = ancount;
        self
    }

    pub fn set_nscount(mut self, nscount: u16) -> Self {
        self.nscount = nscount;
        self
    }

    pub fn set_arcount(mut self, arcount: u16) -> Self {
        self.arcount = arcount;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn test_question_header() {
        let expected_header: [u8; 12] = [
            0x02,
            0x0F,
            // QR, OPCODE, AA, TC, RD
            0b0_0000_0_0_1,
            // RA, Z, RCODE
            0b0_000_0000,
            // QDCOUNT
            0,
            2,
            // ANCOUNT
            0,
            0,
            // NSCOUNT
            0,
            0,
            // ARCOUNT
            0,
            0,
        ];
        let header = Header::builder(MessageType::Question)
            .set_id(0x020F)
            .set_recursion_desired(true)
            .set_qdcount(2)
            .finalize();
        let header_bytes = header.to_bytes();
        assert_eq!(Vec::from(expected_header), header_bytes);
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn test_answer_header() {
        let expected_header: [u8; 12] = [
            // ID
            0x13,
            0xE2,
            // QR, OPCODE, AA, TC, RD
            0b1_0000_1_0_1,
            // RA, Z, RCODE
            0b1_000_0000,
            // QDCOUNT
            0,
            1,
            // ANCOUNT
            0,
            1,
            // NSCOUNT
            0,
            0,
            // ARCOUNT
            0,
            0,
        ];
        let header = Header::builder(MessageType::Answer)
            .set_id(0x13E2)
            .set_authoritative_ans(true)
            .set_recursion_desired(true)
            .set_recursion_available(true)
            .set_qdcount(1)
            .set_ancount(1)
            .finalize();
        let header_bytes = header.to_bytes();
        assert_eq!(Vec::from(expected_header), header_bytes);
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn test_header_construction() {
        let id: u16 = 0x90CB;
        let header = Header::builder(MessageType::Question)
            .set_id(id)
            .set_opcode(QueryOpcode::Query)
            .set_recursion_desired(true)
            .set_qdcount(2)
            .finalize();
        let expected_bytes: [u8; 12] = [
            // ID
            0x90,
            0xCB,
            // QR, OPCODE, AA, TC, RD
            0b0_0000_0_0_1,
            // RA, Z, RCODE
            0b0_000_0000,
            // QDCOUNT
            0,
            2,
            // ANCOUNT
            0,
            0,
            // NSCOUNT
            0,
            0,
            // ARCOUNT
            0,
            0,
        ];

        let header_bytes = header.to_bytes();
        assert_eq!(expected_bytes.to_vec(), header_bytes);

        let id: u16 = 0x2BA2;
        let header = Header::builder(MessageType::Answer)
            .set_id(id)
            .set_opcode(QueryOpcode::Query)
            .set_authoritative_ans(true)
            .set_recursion_desired(true)
            .set_recursion_available(true)
            .set_qdcount(3)
            .set_ancount(3)
            .finalize();
        let expected_bytes: [u8; 12] = [
            // ID
            0x2B,
            0xA2,
            // QR, OPCODE, AA, TC, RD
            0b1_0000_1_0_1,
            // RA, Z, RCODE
            0b1_000_0000,
            // QDCOUNT
            0,
            3,
            // ANCOUNT
            0,
            3,
            // NSCOUNT
            0,
            0,
            // ARCOUNT
            0,
            0,
        ];

        let header_bytes = header.to_bytes();
        assert_eq!(expected_bytes.to_vec(), header_bytes);
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn test_header_parse() {
        let header_bytes: [u8; 12] = [
            // ID
            0x90,
            0xCB,
            // QR, OPCODE, AA, TC, RD
            0b0_0000_0_0_1,
            // RA, Z, RCODE
            0b0_000_0000,
            // QDCOUNT
            0,
            2,
            // ANCOUNT
            0,
            0,
            // NSCOUNT
            0,
            0,
            // ARCOUNT
            0,
            0,
        ];
        let (header, _) = Header::parse(&header_bytes).unwrap();
        assert_eq!(header.id, 0x90CB);
        assert_eq!(header.qr, MessageType::Question);
        assert_eq!(header.opcode, QueryOpcode::Query);
        assert!(!header.authoritative_ans);
        assert!(!header.truncation);
        assert!(header.recursion_desired);
        assert!(!header.recursion_available);
        assert_eq!(header.response_code, ResponseCode::NoError);
        assert_eq!(header.qdcount, 2);
        assert_eq!(header.ancount, 0);
        assert_eq!(header.nscount, 0);
        assert_eq!(header.arcount, 0);

        let header_bytes: [u8; 12] = [
            // ID
            0x2B,
            0xA2,
            // QR, OPCODE, AA, TC, RD
            0b1_0000_1_0_1,
            // RA, Z, RCODE
            0b1_000_0000,
            // QDCOUNT
            0,
            3,
            // ANCOUNT
            0,
            3,
            // NSCOUNT
            0,
            0,
            // ARCOUNT
            0,
            0,
        ];
        let (header, _) = Header::parse(&header_bytes).unwrap();
        assert_eq!(header.id, 0x2BA2);
        assert_eq!(header.qr, MessageType::Answer);
        assert_eq!(header.opcode, QueryOpcode::Query);
        assert!(header.authoritative_ans);
        assert!(!header.truncation);
        assert!(header.recursion_desired);
        assert!(header.recursion_available);
        assert_eq!(header.response_code, ResponseCode::NoError);
        assert_eq!(header.qdcount, 3);
        assert_eq!(header.ancount, 3);
        assert_eq!(header.nscount, 0);
        assert_eq!(header.arcount, 0);
    }
}
