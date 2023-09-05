use std::net::Ipv4Addr;

use crate::BytesSerializable;

/// Hosts that have multiple Internet addresses will have multiple A records.
/// A records cause no additional section processing. The RDATA section of an A line in a master
/// file is an Internet address expressed as four decimal numbers separated by dots without any
/// imbedded spaces (e.g., "10.2.0.52" or "192.0.5.6").
pub struct ARdata {
    /// Support only IPV4 addresses for initial iteration
    address: Ipv4Addr,
}

impl BytesSerializable for ARdata {
    type ParseError = ();

    fn to_bytes(&self) -> Vec<u8> {
        Vec::from(self.address.octets())
    }

    fn parse(_bytes: &[u8]) -> Result<Self, Self::ParseError> {
        todo!()
    }
}
