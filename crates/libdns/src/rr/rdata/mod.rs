use std::num::Wrapping;

use itertools::Itertools;

use crate::{
    domain::DomainName, parse_utils::parse_u32, types::CharacterString, BytesSerializable,
    ParseDataError,
};

pub mod internet;

/// A type representing the data of a `CNAME` resource type.
/// A <domain-name> which specifies the canonical or primary name for the owner.
/// The owner name is an alias.
#[derive(Debug, PartialEq)]
pub struct CnameBytes {
    cname: DomainName,
}

impl BytesSerializable for CnameBytes {
    fn to_bytes(&self) -> Vec<u8> {
        self.cname.to_bytes()
    }

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), ParseDataError> {
        let (cname, remaining_input) = DomainName::parse(bytes)?;
        Ok((Self { cname }, remaining_input))
    }
}

#[derive(Debug, PartialEq)]
pub struct NsdnameBytes {
    nsdname: DomainName,
}

impl NsdnameBytes {
    pub fn new(nsdname: DomainName) -> Self { Self { nsdname } }
}

impl BytesSerializable for NsdnameBytes {
    fn to_bytes(&self) -> Vec<u8> {
        self.nsdname.to_bytes()
    }

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), ParseDataError> {
        let (nsdname, remaining_input) = DomainName::parse(bytes)?;
        Ok((Self { nsdname }, remaining_input))
    }
}

#[derive(Debug, PartialEq)]
pub struct PtrBytes {
    ptrdname: DomainName,
}

impl PtrBytes {
    pub fn new(ptrdname: DomainName) -> Self { Self { ptrdname } }
}

impl BytesSerializable for PtrBytes {
    fn to_bytes(&self) -> Vec<u8> {
        self.ptrdname.to_bytes()
    }

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), ParseDataError> {
        let (ptrdname, remaining_input) = DomainName::parse(bytes)?;
        Ok((Self { ptrdname }, remaining_input))
    }
}

/// SOA records cause no additional section processing. All times are in units of seconds.
/// Most of these fields are pertinent only for name server maintenance operations. However, MINIMUM is used
/// in all query operations that retrieve RRs from a zone. Whenever a RR is sent in a response to a query,
/// the TTL field is set to the maximum of the TTL field from the RR and the MINIMUM field in the appropriate SOA.
/// Thus MINIMUM is a lower bound on the TTL field for all RRs in a zone. Note that this use of MINIMUM should
/// occur when the RRs are copied into the response and not when the zone is loaded from a master file or via a
/// zone transfer. The reason for this provison is to allow future dynamic update facilities to change the SOA
/// RR with known semantics.
#[derive(Debug, PartialEq)]
pub struct SoaBytes {
    /// The <domain-name> of the name server that was the original or primary source of data for this zone.
    mname: DomainName,
    /// A <domain-name> which specifies the mailbox of the person responsible for this zone.
    rname: DomainName,
    /// The unsigned 32 bit version number of the original copy of the zone. Zone transfers preserve this value.
    /// This value wraps and should be compared using sequence space arithmetic.
    serial: Wrapping<u32>,
    /// A 32 bit time interval before the zone should be refreshed.
    refresh: u32,
    /// A 32 bit time interval that should elapse before a failed refresh should be retried.
    retry: u32,
    /// A 32 bit time value that specifies the upper limit on the time interval that can elapse before the zone is no
    /// longer authoritative
    expire: u32,
    /// The unsigned 32 bit minimum TTL field that should be exported with any RR from this zone.
    minimum: u32,
}

impl BytesSerializable for SoaBytes {
    fn to_bytes(&self) -> Vec<u8> {
        [&self.mname, &self.rname]
            .iter()
            .flat_map(|domain_name| domain_name.to_bytes())
            .chain(
                [
                    self.serial.0,
                    self.refresh,
                    self.retry,
                    self.expire,
                    self.minimum,
                ]
                .map(|val| Vec::from(val.to_be_bytes()))
                .into_iter()
                .flatten(),
            )
            .collect_vec()
    }

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), ParseDataError> {
        let (mname, remaining_input) = DomainName::parse(bytes)?;
        let (rname, remaining_input) = DomainName::parse(remaining_input)?;
        let (remaining_input, serial) =
            parse_u32(remaining_input).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (remaining_input, refresh) =
            parse_u32(remaining_input).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (remaining_input, retry) =
            parse_u32(remaining_input).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (remaining_input, expire) =
            parse_u32(remaining_input).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let (remaining_input, minimum) =
            parse_u32(remaining_input).map_err(|_| ParseDataError::InvalidByteStructure)?;
        Ok((
            Self {
                mname,
                rname,
                serial: Wrapping(serial),
                refresh,
                retry,
                expire,
                minimum,
            },
            remaining_input,
        ))
    }
}

/// TXT RRs are used to hold descriptive text. The semantics of the text depends on the domain where it is found.
#[derive(Debug, PartialEq)]
pub struct TxtBytes {
    /// One or more <character-string>s.
    txt_data: Vec<CharacterString>,
}

impl BytesSerializable for TxtBytes {
    fn to_bytes(&self) -> Vec<u8> {
        self.txt_data
            .iter()
            .flat_map(|cs| cs.to_bytes())
            .collect_vec()
    }

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), ParseDataError> {
        let mut bytes = bytes;
        let mut txt_data = Vec::new();
        while let Ok((character_string, remaining_input)) = CharacterString::parse(bytes) {
            txt_data.push(character_string);
            bytes = remaining_input;
        }
        Ok((Self { txt_data }, bytes))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ascii::AsciiString;

    use super::*;

    #[test]
    fn test_parse_cname_bytes() {
        let domain = DomainName::try_from("bing.com").unwrap();
        let expected_bytes = domain.to_bytes();
        let (cname, _) = CnameBytes::parse(&expected_bytes).unwrap();
        assert_eq!(cname.cname, domain);
    }

    #[test]
    fn test_parse_nsdname_bytes() {
        let domain = DomainName::try_from("stackoverflow.com").unwrap();
        let expected_bytes = domain.to_bytes();
        let (nsdname, _) = NsdnameBytes::parse(&expected_bytes).unwrap();
        assert_eq!(nsdname.nsdname, domain);
    }

    #[test]
    fn test_parse_ptr_bytes() {
        let domain = DomainName::try_from("playground.net").unwrap();
        let expected_bytes = domain.to_bytes();
        let (ptrdname, _) = PtrBytes::parse(&expected_bytes).unwrap();
        assert_eq!(ptrdname.ptrdname, domain);
    }

    #[test]
    fn test_serialize_soa_bytes() {
        let mname = DomainName::try_from("ns1.example.com").unwrap();
        let rname = DomainName::try_from("mail.example.com").unwrap();
        let serial = Wrapping(2023113001u32);
        let refresh: u32 = 3600;
        let retry: u32 = 600;
        let expire: u32 = 5184000;
        let minimum: u32 = 60;

        let mut bytes = Vec::new();
        bytes.extend(mname.to_bytes());
        bytes.extend(rname.to_bytes());
        bytes.extend(serial.0.to_be_bytes());
        bytes.extend(refresh.to_be_bytes());
        bytes.extend(retry.to_be_bytes());
        bytes.extend(expire.to_be_bytes());
        bytes.extend(minimum.to_be_bytes());

        let soa = SoaBytes {
            mname: mname.clone(),
            rname: rname.clone(),
            serial,
            refresh,
            retry,
            expire,
            minimum,
        };
        let serialized_bytes = soa.to_bytes();
        assert_eq!(bytes, serialized_bytes)
    }

    #[test]
    fn test_parse_soa_bytes() {
        let mname = DomainName::try_from("ns1.example.com").unwrap();
        let rname = DomainName::try_from("mail.example.com").unwrap();
        let serial = Wrapping(2023113001);
        let refresh = 3600;
        let retry = 600;
        let expire = 5184000;
        let minimum = 60;
        let soa = SoaBytes {
            mname: mname.clone(),
            rname: rname.clone(),
            serial,
            refresh,
            retry,
            expire,
            minimum,
        };

        let expected_bytes = soa.to_bytes();
        let (parsed_soa, _) = SoaBytes::parse(&expected_bytes).unwrap();
        assert_eq!(parsed_soa.mname, mname);
        assert_eq!(parsed_soa.rname, rname);
        assert_eq!(parsed_soa.serial, serial);
        assert_eq!(parsed_soa.refresh, refresh);
        assert_eq!(parsed_soa.retry, retry);
        assert_eq!(parsed_soa.expire, expire);
        assert_eq!(parsed_soa.minimum, minimum);
    }

    #[test]
    fn test_serialize_txt_bytes() {
        let charstr1 = CharacterString::try_from(AsciiString::from_str("En").unwrap()).unwrap();
        let charstr2 = CharacterString::try_from(AsciiString::from_str("Taro").unwrap()).unwrap();
        let charstr3 = CharacterString::try_from(AsciiString::from_str("Adun").unwrap()).unwrap();

        let bytes = charstr1.to_bytes();
        let txt = TxtBytes {
            txt_data: vec![charstr1.clone()],
        };
        let txt_bytes = txt.to_bytes();
        assert_eq!(bytes, txt_bytes);

        let bytes = bytes
            .into_iter()
            .chain(charstr2.to_bytes())
            .chain(charstr3.to_bytes())
            .collect::<Vec<_>>();
        let txt = TxtBytes {
            txt_data: vec![charstr1, charstr2, charstr3],
        };
        let txt_bytes = txt.to_bytes();
        assert_eq!(bytes, txt_bytes);
    }

    #[test]
    fn test_parse_txt_bytes() {
        let charstr1 =
            CharacterString::try_from(AsciiString::from_str("Hesitation").unwrap()).unwrap();
        let charstr2 = CharacterString::try_from(AsciiString::from_str("is").unwrap()).unwrap();
        let charstr3 = CharacterString::try_from(AsciiString::from_str("defeat").unwrap()).unwrap();

        let bytes = charstr1.to_bytes();
        let (txt_bytes, _) = TxtBytes::parse(&bytes).unwrap();
        assert_eq!(txt_bytes.txt_data, vec![charstr1.clone()]);

        let bytes = bytes
            .into_iter()
            .chain(charstr2.to_bytes())
            .chain(charstr3.to_bytes())
            .collect::<Vec<_>>();

        let (txt_bytes, _) = TxtBytes::parse(&bytes).unwrap();
        assert_eq!(txt_bytes.txt_data, vec![charstr1, charstr2, charstr3]);
    }
}
