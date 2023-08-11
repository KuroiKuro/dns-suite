use itertools::Itertools;

use crate::{
    domain::DomainLabel,
    rr::{Qtype, ResourceRecordQClass},
};

/// A struct depicting a question in a DNS message. The question section in the messsage
/// can contain multiple question, all represented by individual `Question` instances.
/// This means that a DNS message with 2 questions will contain 2 `Question` instances
/// packed into bytes
pub struct Question {
    qname: DomainLabel,
    qtype: Qtype,
    qclass: ResourceRecordQClass,
}

impl Question {
    pub fn new(qname: DomainLabel, qtype: Qtype, qclass: ResourceRecordQClass) -> Self {
        Self {
            qname,
            qtype,
            qclass,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let qname = self.qname.as_bytes().to_owned();
        let qtype = (self.qtype as u16).to_be_bytes().to_vec();
        let qclass = (self.qclass as u16).to_be_bytes().to_vec();
        [qname, qtype, qclass]
            .into_iter()
            .flatten()
            .collect_vec()
    }
}
