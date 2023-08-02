pub mod rdata;

use crate::domain::DomainName;

/// An enum of the available resource record types defined in RFC 1035.
/// TYPE fields are used in resource records.  Note that these types are a
/// subset of QTYPEs.

pub enum ResourceRecordType {
    /// A host address
    A = 1,
    /// An authoritative name server
    Ns = 2,
    /// A mail destination (Obsolete - use MX)
    Md = 3,
    /// A mail forwarder (Obsolete - use MX)
    Mf = 4,
    /// The canonical name for an alias
    Cname = 5,
    /// Marks the start of a zone of authority
    Soa = 6,
    /// A mailbox domain name (EXPERIMENTAL)
    Mb = 7,
    /// A mail group member (EXPERIMENTAL)
    Mg = 8,
    /// A mail rename domain name (EXPERIMENTAL)
    Mr = 9,
    /// A null RR (EXPERIMENTAL)
    Null = 10,
    /// A well known service description
    Wks = 11,
    /// A domain name pointer
    Ptr = 12,
    /// Host information
    Hinfo = 13,
    /// Mailbox or mail list information
    Minfo = 14,
    /// Mail exchange
    Mx = 15,
    /// Text strings
    Txt = 16,
}

/// An enum of the available query types defined in RFC 1035.
/// QTYPE fields appear in the question part of a query. QTYPES are a
/// superset of TYPEs, hence all TYPEs are valid QTYPEs.
pub enum Qtype {
    /// A request for a transfer of an entire zone
    Axfr = 252,
    /// A request for mailbox-related records (MB, MG or MR)
    Mailb = 253,
    /// A request for mail agent RRs (Obsolete - see MX)
    Maila = 254,
    /// Represented in the spec as `*`. A request for all records
    All = 255
}

/// CLASS fields appear in resource records
pub enum ResourceRecordClass {
    /// The internet
    In = 1,
    /// the CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    Cs = 2,
    /// The CHAOS class
    Ch = 3,
    /// Hesiod [Dyer 87]
    Hs = 4
}

/// QCLASS fields appear in the question section of a query. QCLASS values
/// are a superset of CLASS values; every CLASS is a valid QCLASS.
pub enum ResourceRecordQClass {
    /// Any class
    All = 255
}



/// All RRs have the same top level format shown below:
///                               1  1  1  1  1  1
/// 0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                                               |
/// /                                               /
/// /                      NAME                     /
/// |                                               |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                      TYPE                     |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                     CLASS                     |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                      TTL                      |
/// |                                               |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                   RDLENGTH                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--|
/// /                     RDATA                     /
/// /                                               /
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
pub struct ResourceRecord {
    /// An owner name, i.e., the name of the node to which this resource record pertains.
    name: DomainName,
    /// Two octets containing one of the RR TYPE codes.
    r#type: ResourceRecordType,
    /// Two octets containing one of the RR CLASS codes.
    class: ResourceRecordClass,
    /// A 32 bit signed integer that specifies the time interval that the resource record
    /// may be cached before the source of the information should again be consulted. Zero
    /// values are interpreted to mean that the RR can only be used for the transaction in
    /// progress, and should not be cached. For example, SOA records are always distributed
    /// with a zero TTL to prohibit caching.  Zero values can also be used for extremely
    /// volatile data.
    ttl: i32,
    /// An unsigned 16 bit integer that specifies the length in octets of the RDATA field.
    rdlength: u16,
    /// A variable length string of octets that describes the resource. The format of this
    /// information varies according to the TYPE and CLASS of the resource record.
    rdata: Vec<u8>,
}
