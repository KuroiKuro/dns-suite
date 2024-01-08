pub mod rdata;

/// An enum of the available resource record types defined in RFC 1035.
/// TYPE fields are used in resource records.  Note that these types are a
/// subset of QTYPEs.
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
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

impl TryFrom<u16> for ResourceRecordType {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ResourceRecordType::A),
            2 => Ok(ResourceRecordType::Ns),
            3 => Ok(ResourceRecordType::Md),
            4 => Ok(ResourceRecordType::Mf),
            5 => Ok(ResourceRecordType::Cname),
            6 => Ok(ResourceRecordType::Soa),
            7 => Ok(ResourceRecordType::Mb),
            8 => Ok(ResourceRecordType::Mg),
            9 => Ok(ResourceRecordType::Mr),
            10 => Ok(ResourceRecordType::Null),
            11 => Ok(ResourceRecordType::Wks),
            12 => Ok(ResourceRecordType::Ptr),
            13 => Ok(ResourceRecordType::Hinfo),
            14 => Ok(ResourceRecordType::Minfo),
            15 => Ok(ResourceRecordType::Mx),
            16 => Ok(ResourceRecordType::Txt),
            _ => Err(()),
        }
    }
}

/// An enum of the available query types defined in RFC 1035.
/// QTYPE fields appear in the question part of a query. QTYPES are a
/// superset of TYPEs, hence all TYPEs are valid QTYPEs.
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum Qtype {
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
    /// A request for a transfer of an entire zone
    Axfr = 252,
    /// A request for mailbox-related records (MB, MG or MR)
    Mailb = 253,
    /// A request for mail agent RRs (Obsolete - see MX)
    Maila = 254,
    /// Represented in the spec as `*`. A request for all records
    All = 255,
}

impl TryFrom<u16> for Qtype {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Qtype::A),
            2 => Ok(Qtype::Ns),
            3 => Ok(Qtype::Md),
            4 => Ok(Qtype::Mf),
            5 => Ok(Qtype::Cname),
            6 => Ok(Qtype::Soa),
            7 => Ok(Qtype::Mb),
            8 => Ok(Qtype::Mg),
            9 => Ok(Qtype::Mr),
            10 => Ok(Qtype::Null),
            11 => Ok(Qtype::Wks),
            12 => Ok(Qtype::Ptr),
            13 => Ok(Qtype::Hinfo),
            14 => Ok(Qtype::Minfo),
            15 => Ok(Qtype::Mx),
            16 => Ok(Qtype::Txt),
            252 => Ok(Qtype::Axfr),
            253 => Ok(Qtype::Mailb),
            254 => Ok(Qtype::Maila),
            255 => Ok(Qtype::All),
            _ => Err(()),
        }
    }
}

/// CLASS fields appear in resource records
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum ResourceRecordClass {
    /// The internet
    In = 1,
    /// the CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    Cs = 2,
    /// The CHAOS class
    Ch = 3,
    /// Hesiod [Dyer 87]
    Hs = 4,
}

impl TryFrom<u16> for ResourceRecordClass {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ResourceRecordClass::In),
            2 => Ok(ResourceRecordClass::Cs),
            3 => Ok(ResourceRecordClass::Ch),
            4 => Ok(ResourceRecordClass::Hs),
            _ => Err(()),
        }
    }
}

/// QCLASS fields appear in the question section of a query. QCLASS values
/// are a superset of CLASS values; every CLASS is a valid QCLASS.
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum ResourceRecordQClass {
    /// The internet
    In = 1,
    /// the CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    Cs = 2,
    /// The CHAOS class
    Ch = 3,
    /// Hesiod [Dyer 87]
    Hs = 4,
    /// Any class
    All = 255,
}

impl TryFrom<u16> for ResourceRecordQClass {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ResourceRecordQClass::In),
            2 => Ok(ResourceRecordQClass::Cs),
            3 => Ok(ResourceRecordQClass::Ch),
            4 => Ok(ResourceRecordQClass::Hs),
            255 => Ok(ResourceRecordQClass::All),
            _ => Err(()),
        }
    }
}
