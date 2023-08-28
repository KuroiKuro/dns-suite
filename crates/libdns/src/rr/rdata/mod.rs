use std::num::Wrapping;

use itertools::Itertools;

use crate::{domain::DomainName, types::CharacterString};

pub mod internet;

/// A trait to be implemented by a resource type
pub trait Rdata {
    fn to_bytes(&self) -> Vec<u8>;
}

/// A type representing the data of a `CNAME` resource type.
/// A <domain-name> which specifies the canonical or primary name for the owner.
/// The owner name is an alias.
pub struct CnameRdata {
    cname: DomainName,
}

impl Rdata for CnameRdata {
    fn to_bytes(&self) -> Vec<u8> {
        self.cname.to_bytes()
    }
}

pub struct NsdnameRdata {
    nsdname: DomainName,
}

impl Rdata for NsdnameRdata {
    fn to_bytes(&self) -> Vec<u8> {
        self.nsdname.to_bytes()
    }
}

pub struct PtrRdata {
    ptrdname: DomainName,
}

impl Rdata for PtrRdata {
    fn to_bytes(&self) -> Vec<u8> {
        self.ptrdname.to_bytes()
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
pub struct SoaRdata {
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

impl Rdata for SoaRdata {
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
}

/// TXT RRs are used to hold descriptive text. The semantics of the text depends on the domain where it is found.
pub struct TxtRdata {
    /// One or more <character-string>s.
    txt_data: Vec<CharacterString>,
}

impl Rdata for TxtRdata {
    fn to_bytes(&self) -> Vec<u8> {
        self.txt_data
            .iter()
            .flat_map(|cs| cs.to_bytes())
            .collect_vec()
    }
}
