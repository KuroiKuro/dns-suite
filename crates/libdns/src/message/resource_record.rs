use crate::{
    domain::DomainName,
    parse_utils::{byte_parser, parse_i32, parse_u16},
    rr::{rdata::{self, internet::ARdata, NsdnameBytes, CnameBytes, SoaBytes, PtrBytes, TxtBytes}, ResourceRecordClass, ResourceRecordType},
    BytesSerializable, ParseDataError,
};

/// An enum to represent all of the possible forms data that can be included in a resource record.
/// An enum is used so that we can contain different structs in the `ResourceRecord` struct.
enum Rdata {
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
            },
            ResourceRecordType::Ns => {
                let data = match NsdnameBytes::parse(bytes) {
                    Ok(d) => d.0,
                    _ => return None,
                };
                Some(Self::Ns(data))
            },
            ResourceRecordType::Cname => {
                let data = match CnameBytes::parse(bytes) {
                    Ok(d) => d.0,
                    _ => return None,
                };
                Some(Self::Cname(data))
            },
            ResourceRecordType::Soa => {
                let data = match SoaBytes::parse(bytes) {
                    Ok(d) => d.0,
                    _ => return None,
                };
                Some(Self::Soa(data))
            },
            // ResourceRecordType::Wks => todo!(),
            ResourceRecordType::Ptr => {
                let data = match PtrBytes::parse(bytes) {
                    Ok(d) => d.0,
                    _ => return None,
                };
                Some(Self::Ptr(data))
            },
            // ResourceRecordType::Hinfo => todo!(),
            // ResourceRecordType::Mx => todo!(),
            ResourceRecordType::Txt => {
                let data = match TxtBytes::parse(bytes) {
                    Ok(d) => d.0,
                    _ => return None,
                };
                Some(Self::Txt(data))
            },
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
        let rdata = Rdata::parse(r#type, rdata_bytes).ok_or(ParseDataError::InvalidByteStructure)?;

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
    use std::net::Ipv4Addr;
    use itertools::Itertools;

    use super::*;

    /// Create the expected bytes for the initial section of the resource record, which is common across all of the testing
    /// functions here and does not require any special logic
    fn create_expected_bytes(name: &DomainName, r#type: ResourceRecordType, class: ResourceRecordClass, ttl: i32, rdlength: usize) -> Vec<u8> {
        // Use a separate buffer for type, class and ttl because we always know the number of bytes for them
        let mut bytes = Vec::with_capacity(8);
        bytes.extend((r#type as u16).to_be_bytes());
        bytes.extend((class as u16).to_be_bytes());
        bytes.extend(ttl.to_be_bytes());
        bytes.extend((rdlength as u16).to_be_bytes());

        name
            .to_bytes()
            .into_iter()
            .chain(bytes)
            .collect_vec()
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

        let name = DomainName::try_from("example.com").unwrap();
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
}
