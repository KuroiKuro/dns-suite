use crate::{domain::DomainName, rr::{ResourceRecordType, ResourceRecordClass}};

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