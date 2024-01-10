use crate::{
    domain::DomainName,
    parse_utils::{byte_parser, parse_i32, parse_u16},
    rr::{
        rdata::{self, internet::ARdata, CnameBytes, NsdnameBytes, PtrBytes, SoaBytes, TxtBytes},
        ResourceRecordClass, ResourceRecordType,
    },
    BytesSerializable, ParseDataError,
};

/// An enum to represent all of the possible forms data that can be included in a resource record.
/// An enum is used so that we can contain different structs in the `ResourceRecord` struct.
#[derive(Debug, PartialEq)]
pub enum Rdata {
    Cname(rdata::CnameBytes),
    Ns(rdata::NsdnameBytes),
    Ptr(rdata::PtrBytes),
    Soa(rdata::SoaBytes),
    Txt(rdata::TxtBytes),
    A(rdata::internet::ARdata),
}

impl Rdata {
    /// Serializes to bytes. We cannot use the `ByteSerializable` trait because the `parse` function requires
    /// a different function signature
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Rdata::Cname(data) => data.to_bytes(),
            Rdata::Ns(data) => data.to_bytes(),
            Rdata::Ptr(data) => data.to_bytes(),
            Rdata::Soa(data) => data.to_bytes(),
            Rdata::Txt(data) => data.to_bytes(),
            Rdata::A(data) => data.to_bytes(),
        }
    }

    /// Parse from bytes. We cannot use the `ByteSerializable` trait because the `parse` function requires
    /// a different function signature
    pub fn parse(r#type: ResourceRecordType, bytes: &[u8]) -> Option<Self> {
        match r#type {
            ResourceRecordType::A => {
                let data = match ARdata::parse(bytes) {
                    Ok(d) => d.0,
                    Err(_) => return None,
                };
                Some(Self::A(data))
            }
            ResourceRecordType::Ns => {
                let data = match NsdnameBytes::parse(bytes) {
                    Ok(d) => d.0,
                    _ => return None,
                };
                Some(Self::Ns(data))
            }
            ResourceRecordType::Cname => {
                let data = match CnameBytes::parse(bytes) {
                    Ok(d) => d.0,
                    _ => return None,
                };
                Some(Self::Cname(data))
            }
            ResourceRecordType::Soa => {
                let data = match SoaBytes::parse(bytes) {
                    Ok(d) => d.0,
                    _ => return None,
                };
                Some(Self::Soa(data))
            }
            // ResourceRecordType::Wks => todo!(),
            ResourceRecordType::Ptr => {
                let data = match PtrBytes::parse(bytes) {
                    Ok(d) => d.0,
                    _ => return None,
                };
                Some(Self::Ptr(data))
            }
            // ResourceRecordType::Hinfo => todo!(),
            // ResourceRecordType::Mx => todo!(),
            ResourceRecordType::Txt => {
                let data = match TxtBytes::parse(bytes) {
                    Ok(d) => d.0,
                    _ => return None,
                };
                Some(Self::Txt(data))
            }
            _ => None,
        }
    }
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
/// The struct contains most of the fields shown in the diagram, with the
/// exception of rdlength, which is computed on the fly when the struct is
/// serialized.
///
/// For reference, `rdlength` is defined as follows:
/// An unsigned 16 bit integer that specifies the length in octets of the RDATA field.
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
    /// A variable length string of octets that describes the resource. The format of this
    /// information varies according to the TYPE and CLASS of the resource record.
    rdata: Rdata,
}

impl ResourceRecord {
    pub fn new(
        name: DomainName,
        r#type: ResourceRecordType,
        class: ResourceRecordClass,
        ttl: i32,
        rdata: Rdata,
    ) -> Self {
        Self {
            name,
            r#type,
            class,
            ttl,
            rdata,
        }
    }
}

impl BytesSerializable for ResourceRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.name.to_bytes();
        bytes.extend((self.r#type as u16).to_be_bytes());
        bytes.extend((self.class as u16).to_be_bytes());
        bytes.extend(self.ttl.to_be_bytes());
        let rdata_bytes = self.rdata.to_bytes();
        let rdlength = rdata_bytes.len() as u16;
        bytes.extend(rdlength.to_be_bytes());
        bytes.extend(rdata_bytes);
        bytes
    }

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), ParseDataError>
    where
        Self: std::marker::Sized,
    {
        let (domain_name, remaining_input) = DomainName::parse(bytes)?;
        let (remaining_input, parsed_type_bytes) =
            byte_parser(remaining_input, 2).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (_, type_data) =
            parse_u16(parsed_type_bytes).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let r#type = ResourceRecordType::try_from(type_data)
            .map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (remaining_input, parsed_class_bytes) =
            byte_parser(remaining_input, 2).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (_, class_data) =
            parse_u16(parsed_class_bytes).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let class = ResourceRecordClass::try_from(class_data)
            .map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (remaining_input, parsed_ttl_bytes) =
            byte_parser(remaining_input, 4).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (_, ttl_data) =
            parse_i32(parsed_ttl_bytes).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (remaining_input, parsed_rdlength_bytes) =
            byte_parser(remaining_input, 2).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (_, rdlength) =
            parse_u16(parsed_rdlength_bytes).map_err(|_| ParseDataError::InvalidByteStructure)?;

        // Based on rdlength, take the exact amount of data to parse for rdata
        let (remaining_input, rdata_bytes) = byte_parser(remaining_input, rdlength as usize)
            .map_err(|_| ParseDataError::InvalidByteStructure)?;
        let rdata =
            Rdata::parse(r#type, rdata_bytes).ok_or(ParseDataError::InvalidByteStructure)?;

        Ok((
            Self {
                class,
                name: domain_name,
                rdata,
                ttl: ttl_data,
                r#type,
            },
            remaining_input,
        ))
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use std::net::Ipv4Addr;

    use super::*;

    const EXAMPLE_DOMAIN_BYTES: [u8; 13] = [
        7, 101, 120, 97, 109, 112, 108, 101, 3, 99, 111, 109, 0
    ];
    const EXAMPLE_DOMAIN: &str = "example.com";

    /// Create the expected bytes for the initial section of the resource record, which is common across all of the testing
    /// functions here and does not require any special logic
    fn create_expected_bytes(
        name: &DomainName,
        r#type: ResourceRecordType,
        class: ResourceRecordClass,
        ttl: i32,
        rdlength: usize,
    ) -> Vec<u8> {
        // Use a separate buffer for type, class and ttl because we always know the number of bytes for them
        let mut bytes = Vec::with_capacity(8);
        bytes.extend((r#type as u16).to_be_bytes());
        bytes.extend((class as u16).to_be_bytes());
        bytes.extend(ttl.to_be_bytes());
        bytes.extend((rdlength as u16).to_be_bytes());

        name.to_bytes().into_iter().chain(bytes).collect_vec()
    }

    #[test]
    fn test_resource_record_a_to_bytes() {
        // Test with a simple A record
        let octets = [111, 2, 0, 41];
        let address = Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]);
        let ardata = ARdata::new(address);
        let ardata_bytes = ardata.to_bytes();
        let rdlength = ardata_bytes.len();
        let rdata = Rdata::A(ardata);

        let name = DomainName::try_from(EXAMPLE_DOMAIN).unwrap();
        let r#type = ResourceRecordType::A;
        let class = ResourceRecordClass::In;
        let ttl = 1132;

        // Create expected bytes
        let mut expected_bytes = create_expected_bytes(&name, r#type, class, ttl, rdlength);
        expected_bytes.extend(ardata_bytes);

        let rr = ResourceRecord::new(name, r#type, class, ttl, rdata);
        let bytes = rr.to_bytes();
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_resource_record_a_parse() {
        let mut bytes_to_parse = Vec::from(EXAMPLE_DOMAIN_BYTES);
        let expected_rr_type = ResourceRecordType::A;
        let expected_rr_class = ResourceRecordClass::In;
        let expected_ttl: i32 = 86400;
        let expected_domain = DomainName::try_from(EXAMPLE_DOMAIN).unwrap();

        let octets = [100, 201, 192, 61];
        let expected_ardata = ARdata::new(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]));
        let expected_ardata_bytes = expected_ardata.to_bytes();

        bytes_to_parse.extend((expected_rr_type as u16).to_be_bytes());
        bytes_to_parse.extend((expected_rr_class as u16).to_be_bytes());
        bytes_to_parse.extend(expected_ttl.to_be_bytes());
        bytes_to_parse.extend((expected_ardata_bytes.len() as u16).to_be_bytes());
        bytes_to_parse.extend(expected_ardata.to_bytes());

        let (rr, remaining_bytes) = ResourceRecord::parse(&bytes_to_parse).unwrap();
        assert!(remaining_bytes.is_empty());
        assert_eq!(rr.name, expected_domain);
        assert_eq!(rr.r#type, expected_rr_type);
        assert_eq!(rr.class, expected_rr_class);
        assert_eq!(rr.ttl, expected_ttl);
        assert_eq!(rr.rdata, Rdata::A(expected_ardata));
    }

    #[test]
    fn test_resource_record_ns_to_bytes() {
        let ns_name = "ns.example.com";
        let domain_name = DomainName::try_from(ns_name).unwrap();
        let ns = NsdnameBytes::new(domain_name);
        let ns_bytes = ns.to_bytes();
        let rdlength = ns_bytes.len();
        let rdata = Rdata::Ns(ns);

        let name = DomainName::try_from(EXAMPLE_DOMAIN).unwrap();
        let r#type = ResourceRecordType::Ns;
        let class = ResourceRecordClass::In;
        let ttl = 11932;

        // Create expected bytes
        let mut expected_bytes = create_expected_bytes(&name, r#type, class, ttl, rdlength);
        expected_bytes.extend(ns_bytes);

        let rr = ResourceRecord::new(name, r#type, class, ttl, rdata);
        let bytes = rr.to_bytes();
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_resource_record_ns_parse() {
        // Add bytes for `ns` label in `ns.example.com`
        let mut bytes_to_parse = Vec::from(EXAMPLE_DOMAIN_BYTES);
        let expected_rr_type = ResourceRecordType::Ns;
        let expected_rr_class = ResourceRecordClass::In;
        let expected_ttl: i32 = 86400;
        let expected_domain = DomainName::try_from(EXAMPLE_DOMAIN).unwrap();

        let ns_domain = "ns.example.com";
        let expected_ns_domain = DomainName::try_from(ns_domain).unwrap();
        let expected_ns = NsdnameBytes::new(expected_ns_domain);
        let expected_ns_bytes = expected_ns.to_bytes();

        bytes_to_parse.extend((expected_rr_type as u16).to_be_bytes());
        bytes_to_parse.extend((expected_rr_class as u16).to_be_bytes());
        bytes_to_parse.extend(expected_ttl.to_be_bytes());
        bytes_to_parse.extend((expected_ns_bytes.len() as u16).to_be_bytes());
        bytes_to_parse.extend(expected_ns.to_bytes());

        let (rr, remaining_bytes) = ResourceRecord::parse(&bytes_to_parse).unwrap();
        assert!(remaining_bytes.is_empty());
        assert_eq!(rr.name, expected_domain);
        assert_eq!(rr.r#type, expected_rr_type);
        assert_eq!(rr.class, expected_rr_class);
        assert_eq!(rr.ttl, expected_ttl);
        assert_eq!(rr.rdata, Rdata::Ns(expected_ns));
    }

    #[test]
    fn test_resource_record_ptr_to_bytes() {
        let subdomain = "sub.example.com";
        let subdomain_name = DomainName::try_from(subdomain).unwrap();
        let ptr = PtrBytes::new(subdomain_name);
        let ptr_bytes = ptr.to_bytes();
        let rdlength = ptr_bytes.len();
        let rdata = Rdata::Ptr(ptr);

        let name = DomainName::try_from(EXAMPLE_DOMAIN).unwrap();
        let r#type = ResourceRecordType::Ptr;
        let class = ResourceRecordClass::In;
        let ttl = 11932;

        // Create expected bytes
        let mut expected_bytes = create_expected_bytes(&name, r#type, class, ttl, rdlength);
        expected_bytes.extend(ptr_bytes);

        let rr = ResourceRecord::new(name, r#type, class, ttl, rdata);
        let bytes = rr.to_bytes();
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_resource_record_ptr_parse() {
        // Add bytes for `ns` label in `ns.example.com`
        let mut bytes_to_parse = Vec::from(EXAMPLE_DOMAIN_BYTES);
        let expected_rr_type = ResourceRecordType::Ptr;
        let expected_rr_class = ResourceRecordClass::In;
        let expected_ttl: i32 = 86400;
        let expected_domain = DomainName::try_from(EXAMPLE_DOMAIN).unwrap();

        let ptr_domain = "sub.example.com";
        let expected_ptr_domain = DomainName::try_from(ptr_domain).unwrap();
        let expected_ptr = PtrBytes::new(expected_ptr_domain);
        let expected_ptr_bytes = expected_ptr.to_bytes();

        bytes_to_parse.extend((expected_rr_type as u16).to_be_bytes());
        bytes_to_parse.extend((expected_rr_class as u16).to_be_bytes());
        bytes_to_parse.extend(expected_ttl.to_be_bytes());
        bytes_to_parse.extend((expected_ptr_bytes.len() as u16).to_be_bytes());
        bytes_to_parse.extend(expected_ptr.to_bytes());

        let (rr, remaining_bytes) = ResourceRecord::parse(&bytes_to_parse).unwrap();
        assert!(remaining_bytes.is_empty());
        assert_eq!(rr.name, expected_domain);
        assert_eq!(rr.r#type, expected_rr_type);
        assert_eq!(rr.class, expected_rr_class);
        assert_eq!(rr.ttl, expected_ttl);
        assert_eq!(rr.rdata, Rdata::Ptr(expected_ptr));
    }

    #[test]
    fn test_resource_record_cname_to_bytes() {
        let cname_domain = "cname.example.com";
        let cname_domain_name = DomainName::try_from(cname_domain).unwrap();
        let cname = CnameBytes::new(cname_domain_name);
        let cname_bytes = cname.to_bytes();
        let rdlength = cname_bytes.len();
        let rdata = Rdata::Cname(cname);

        let name = DomainName::try_from(EXAMPLE_DOMAIN).unwrap();
        let r#type = ResourceRecordType::Cname;
        let class = ResourceRecordClass::In;
        let ttl = 21274;

        // Create expected bytes
        let mut expected_bytes = create_expected_bytes(&name, r#type, class, ttl, rdlength);
        expected_bytes.extend(cname_bytes);

        let rr = ResourceRecord::new(name, r#type, class, ttl, rdata);
        let bytes = rr.to_bytes();
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_resource_record_cname_parse() {
        // Add bytes for `ns` label in `ns.example.com`
        let mut bytes_to_parse = Vec::from(EXAMPLE_DOMAIN_BYTES);
        let expected_rr_type = ResourceRecordType::Cname;
        let expected_rr_class = ResourceRecordClass::In;
        let expected_ttl: i32 = 86400;
        let expected_domain = DomainName::try_from(EXAMPLE_DOMAIN).unwrap();

        let cname_domain = "cname.example.com";
        let expected_cname_domain = DomainName::try_from(cname_domain).unwrap();
        let expected_cname = CnameBytes::new(expected_cname_domain);
        let expected_cname_bytes = expected_cname.to_bytes();

        bytes_to_parse.extend((expected_rr_type as u16).to_be_bytes());
        bytes_to_parse.extend((expected_rr_class as u16).to_be_bytes());
        bytes_to_parse.extend(expected_ttl.to_be_bytes());
        bytes_to_parse.extend((expected_cname_bytes.len() as u16).to_be_bytes());
        bytes_to_parse.extend(expected_cname.to_bytes());

        let (rr, remaining_bytes) = ResourceRecord::parse(&bytes_to_parse).unwrap();
        assert!(remaining_bytes.is_empty());
        assert_eq!(rr.name, expected_domain);
        assert_eq!(rr.r#type, expected_rr_type);
        assert_eq!(rr.class, expected_rr_class);
        assert_eq!(rr.ttl, expected_ttl);
        assert_eq!(rr.rdata, Rdata::Cname(expected_cname));
    }

    #[test]
    fn test_resource_record_soa_to_bytes() {
        let mname = "ns.example.com";
        let maname_domain = DomainName::try_from(mname).unwrap();
        let rname = "hostmaster.example.com";
        let rname_domain = DomainName::try_from(rname).unwrap();
        let serial = 2024011001;
        let refresh = 3600;
        let retry = 300;
        let expire = 1814400;
        let minimum = 600;

        let soa = SoaBytes::new(maname_domain, rname_domain, serial, refresh, retry, expire, minimum);
        let soa_bytes = soa.to_bytes();
        let rdlength = soa_bytes.len();
        let rdata = Rdata::Soa(soa);

        let name = DomainName::try_from(EXAMPLE_DOMAIN).unwrap();
        let r#type = ResourceRecordType::Soa;
        let class = ResourceRecordClass::In;
        let ttl = 21274;

        // Create expected bytes
        let mut expected_bytes = create_expected_bytes(&name, r#type, class, ttl, rdlength);
        expected_bytes.extend(soa_bytes);

        let rr = ResourceRecord::new(name, r#type, class, ttl, rdata);
        let bytes = rr.to_bytes();
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_resource_record_soa_parse() {
        // Add bytes for `ns` label in `ns.example.com`
        let mut bytes_to_parse = Vec::from(EXAMPLE_DOMAIN_BYTES);
        let expected_rr_type = ResourceRecordType::Soa;
        let expected_rr_class = ResourceRecordClass::In;
        let expected_ttl: i32 = 86400;
        let expected_domain = DomainName::try_from(EXAMPLE_DOMAIN).unwrap();

        let mname = "ns.example.com";
        let maname_domain = DomainName::try_from(mname).unwrap();
        let rname = "hostmaster.example.com";
        let rname_domain = DomainName::try_from(rname).unwrap();
        let serial = 2024011001;
        let refresh = 3600;
        let retry = 300;
        let expire = 1814400;
        let minimum = 600;
        let expected_soa = SoaBytes::new(maname_domain, rname_domain, serial, refresh, retry, expire, minimum);
        let expected_soa_bytes = expected_soa.to_bytes();
        
        bytes_to_parse.extend((expected_rr_type as u16).to_be_bytes());
        bytes_to_parse.extend((expected_rr_class as u16).to_be_bytes());
        bytes_to_parse.extend(expected_ttl.to_be_bytes());
        bytes_to_parse.extend((expected_soa_bytes.len() as u16).to_be_bytes());
        bytes_to_parse.extend(expected_soa.to_bytes());

        let (rr, remaining_bytes) = ResourceRecord::parse(&bytes_to_parse).unwrap();
        assert!(remaining_bytes.is_empty());
        assert_eq!(rr.name, expected_domain);
        assert_eq!(rr.r#type, expected_rr_type);
        assert_eq!(rr.class, expected_rr_class);
        assert_eq!(rr.ttl, expected_ttl);
        assert_eq!(rr.rdata, Rdata::Soa(expected_soa));
    }
}
