use std::net::Ipv4Addr;

use crate::{parse_utils::byte_parser, BytesSerializable, ParseDataError};

/// Hosts that have multiple Internet addresses will have multiple A records.
/// A records cause no additional section processing. The RDATA section of an A line in a master
/// file is an Internet address expressed as four decimal numbers separated by dots without any
/// imbedded spaces (e.g., "10.2.0.52" or "192.0.5.6").
#[derive(Clone, PartialEq, Debug)]
pub struct ARdata {
    /// Support only IPV4 addresses for initial iteration
    address: Ipv4Addr,
}

impl ARdata {
    pub fn new(address: Ipv4Addr) -> Self {
        Self { address }
    }
}

impl BytesSerializable for ARdata {
    fn to_bytes(&self) -> Vec<u8> {
        Vec::from(self.address.octets())
    }

    fn parse(bytes: &[u8], _parse_count: Option<u16>) -> Result<(Self, &[u8]), ParseDataError> {
        let (remaining_input, parsed_bytes) =
            byte_parser(bytes, 4).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let ardata = Self {
            address: Ipv4Addr::new(
                parsed_bytes[0],
                parsed_bytes[1],
                parsed_bytes[2],
                parsed_bytes[3],
            ),
        };
        Ok((ardata, remaining_input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_ardata_to_bytes() {
        let octets = [132, 142, 0, 212];
        let address = Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]);
        let ardata = ARdata::new(address);
        let bytes = ardata.to_bytes();
        assert_eq!(bytes, octets);
    }

    #[test]
    fn test_ardata_parse() {
        let bytes = [213, 12, 108, 95];
        let (ardata, _) = ARdata::parse(&bytes, None).unwrap();
        let expected_addr = Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
        assert_eq!(ardata.address, expected_addr);
    }
}
