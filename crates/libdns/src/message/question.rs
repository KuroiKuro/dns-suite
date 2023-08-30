
use itertools::Itertools;

use crate::{
    domain::DomainName,
    rr::{Qtype, ResourceRecordQClass}, BytesSerializable, CompressedBytesSerializable,
};

/// A struct depicting a question in a DNS message. The question section in the messsage
/// can contain multiple question, all represented by individual `Question` instances.
/// This means that a DNS message with 2 questions will contain 2 `Question` instances
/// packed into bytes
pub struct Question {
    qname: DomainName,
    qtype: Qtype,
    qclass: ResourceRecordQClass,
}

impl Question {
    pub fn new(qname: DomainName, qtype: Qtype, qclass: ResourceRecordQClass) -> Self {
        Self {
            qname,
            qtype,
            qclass,
        }
    }

    pub fn qname(&self) -> &DomainName {
        &self.qname
    }

    pub fn qtype(&self) -> Qtype {
        self.qtype
    }

    pub fn qclass(&self) -> ResourceRecordQClass {
        self.qclass
    }
}

impl BytesSerializable for Question {
    type ParseError = ();

    fn to_bytes(&self) -> Vec<u8> {
        let qname = self.qname.to_bytes();
        let qtype = (self.qtype as u16).to_be_bytes().to_vec();
        let qclass = (self.qclass as u16).to_be_bytes().to_vec();
        [qname, qtype, qclass]
            .into_iter()
            .flatten()
            .collect_vec()
    }

    fn parse(bytes: &[u8]) -> Result<Self, Self::ParseError> where Self: std::marker::Sized {
        todo!()
    }
}

impl CompressedBytesSerializable for Question {
    type ParseError = ();

    fn to_bytes_compressed(&self, base_offset: u16, label_map: &mut crate::LabelMap) -> (Vec<u8>, u16) {
        let (compressed_domain_bytes, changed_offset) = self.qname.to_bytes_compressed(base_offset, label_map);
        let bytes = compressed_domain_bytes
            .into_iter()
            .chain((self.qtype as u16).to_be_bytes())
            .chain((self.qclass as u16).to_be_bytes())
            .collect_vec();
        let new_offset = changed_offset + (bytes.len() as u16);
        (bytes, new_offset)
    }

    fn parse_compressed(bytes: &[u8], base_offset: u16, label_map: &mut crate::LabelMap) -> (Result<Self, Self::ParseError>, u16) where Self: std::marker::Sized {
        todo!()
    }
}

pub struct MessageQuestions {
    questions: Vec<Question>
}

impl MessageQuestions {
    pub fn new(questions: Vec<Question>) -> Self {
        Self { questions }
    }
}

impl BytesSerializable for MessageQuestions {
    type ParseError = ();

    fn to_bytes(&self) -> Vec<u8> {
        self.questions
            .iter()
            .flat_map(|question| question.to_bytes())
            .collect_vec()
    }

    fn parse(bytes: &[u8]) -> Result<Self, Self::ParseError> where Self: std::marker::Sized {
        todo!()
    }
}

impl CompressedBytesSerializable for MessageQuestions {
    type ParseError = ();

    fn to_bytes_compressed(&self, base_offset: u16, label_map: &mut crate::LabelMap) -> (Vec<u8>, u16) {
        let mut rolling_offset = base_offset;
        let question_bytes = self.questions
            .iter()
            .flat_map(|question| {
                let (bytes, offset) = question.to_bytes_compressed(rolling_offset, label_map);
                rolling_offset = offset;
                bytes
            })
            .collect_vec();
        (question_bytes, rolling_offset)
    }

    fn parse_compressed(bytes: &[u8], base_offset: u16, label_map: &mut crate::LabelMap) -> (Result<Self, Self::ParseError>, u16) where Self: std::marker::Sized {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_to_bytes() {
        let qname = DomainName::try_from("sheets.google.com").unwrap();
        let qtype = Qtype::A;
        let qclass = ResourceRecordQClass::In;
        let question = Question::new(qname.clone(), qtype, qclass);

        let expected_bytes = [
            qname.to_bytes(),
            (qtype as u16).to_be_bytes().to_vec(),
            (qclass as u16).to_be_bytes().to_vec(),
        ]
            .into_iter()
            .flatten()
            .collect_vec();
        let bytes = question.to_bytes();
        assert_eq!(bytes, expected_bytes);

        let qname = DomainName::try_from("audi.com").unwrap();
        let qtype = Qtype::A;
        let qclass = ResourceRecordQClass::In;
        let question = Question::new(qname.clone(), qtype, qclass);

        let expected_bytes = [
            qname.to_bytes(),
            (qtype as u16).to_be_bytes().to_vec(),
            (qclass as u16).to_be_bytes().to_vec(),
        ]
            .into_iter()
            .flatten()
            .collect_vec();
        let bytes = question.to_bytes();
        assert_eq!(bytes, expected_bytes);
    }
}
