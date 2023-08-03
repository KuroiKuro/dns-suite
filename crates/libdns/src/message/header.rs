use super::{MessageType, QueryOpcode, ResponseCode};

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
    opcode: Option<QueryOpcode>,
    /// Defaults to `false`
    authoritative_ans: Option<bool>,
    /// Defaults to `false`
    truncation: Option<bool>,
    /// Defaults to `false`
    recursion_desired: Option<bool>,
    /// Defaults to `false`
    recursion_available: Option<bool>,
    /// Defaults to `ResponseCode::NoError`
    response_code: Option<ResponseCode>,
    /// Defaults to `0`
    qdcount: Option<u16>,
    /// Defaults to `0`
    ancount: Option<u16>,
    /// Defaults to `0`
    nscount: Option<u16>,
    /// Defaults to `0`
    arcount: Option<u16>,
}