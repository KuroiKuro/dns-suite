use itertools::Itertools;

use crate::{
    domain::DomainName, parse_utils::parse_u16, rr::{Qtype, ResourceRecordQClass}, BytesSerializable, CompressedBytesSerializable, MessageOffset, ParseDataError, SerializeCompressedOutcome
};

/// A struct depicting a question in a DNS message. The question section in the messsage
/// can contain multiple questions, all represented by individual `Question` instances.
/// This means that a DNS message with 2 questions will contain 2 `Question` instances
/// packed into bytes
#[derive(Clone, Debug, PartialEq)]
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
    fn to_bytes(&self) -> Vec<u8> {
        let qname = self.qname.to_bytes();
        let qtype = (self.qtype as u16).to_be_bytes().to_vec();
        let qclass = (self.qclass as u16).to_be_bytes().to_vec();
        [qname, qtype, qclass].into_iter().flatten().collect_vec()
    }

    fn parse(bytes: &[u8], parse_count: Option<u16>) -> Result<(Self, &[u8]), ParseDataError>
    where
        Self: std::marker::Sized,
    {
        let (qname, remaining_input) = DomainName::parse(bytes, None).map_err(|_| ParseDataError::InvalidByteStructure)?;

        let (remaining_input, qtype_bytes) = parse_u16(remaining_input).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let qtype = Qtype::try_from(qtype_bytes).map_err(|_| ParseDataError::InvalidByteStructure)?;

        let (remaining_input, qclass_bytes) = parse_u16(remaining_input).map_err(|_| ParseDataError::InvalidByteStructure)?;
        let qclass = ResourceRecordQClass::try_from(qclass_bytes).map_err(|_| ParseDataError::InvalidByteStructure)?;
        Ok((Self::new(qname, qtype, qclass), remaining_input))
    }
}

impl CompressedBytesSerializable for Question {
    fn to_bytes_compressed(
        &self,
        base_offset: u16,
        label_map: &mut crate::LabelMap,
    ) -> SerializeCompressedOutcome {
        let result = self.qname.to_bytes_compressed(base_offset, label_map);
        let compressed_bytes = result
            .compressed_bytes
            .into_iter()
            .chain((self.qtype as u16).to_be_bytes())
            .chain((self.qclass as u16).to_be_bytes())
            .collect_vec();

        // Add 4 which is the number of bytes of qtype and qclass added together
        let new_offset = result.new_offset + 4;
        SerializeCompressedOutcome {
            compressed_bytes,
            new_offset,
        }
    }

    fn parse_compressed(
        _full_message_bytes: &[u8],
        _base_offset: crate::MessageOffset,
    ) -> Result<(Self, crate::MessageOffset), ParseDataError>
    where
        Self: std::marker::Sized,
    {
        todo!()
    }
}

pub struct MessageQuestions {
    questions: Vec<Question>,
}

impl MessageQuestions {
    pub fn new(questions: Vec<Question>) -> Self {
        Self { questions }
    }
}

impl BytesSerializable for MessageQuestions {
    fn to_bytes(&self) -> Vec<u8> {
        self.questions
            .iter()
            .flat_map(|question| question.to_bytes())
            .collect_vec()
    }

    fn parse(bytes: &[u8], parse_count: Option<u16>) -> Result<(Self, &[u8]), ParseDataError>
    where
        Self: std::marker::Sized,
    {
        todo!()
    }
}

impl CompressedBytesSerializable for MessageQuestions {
    fn to_bytes_compressed(
        &self,
        base_offset: u16,
        label_map: &mut crate::LabelMap,
    ) -> SerializeCompressedOutcome {
        let mut rolling_offset = base_offset;
        let question_bytes = self
            .questions
            .iter()
            .flat_map(|question| {
                let result = question.to_bytes_compressed(rolling_offset, label_map);
                rolling_offset = result.new_offset;
                result.compressed_bytes
            })
            .collect_vec();
        SerializeCompressedOutcome {
            compressed_bytes: question_bytes,
            new_offset: rolling_offset,
        }
    }

    fn parse_compressed(
        _full_message_bytes: &[u8],
        _base_offset: MessageOffset,
    ) -> Result<(Self, MessageOffset), ParseDataError>
    where
        Self: std::marker::Sized,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{create_pointer, domain::DomainLabel, LabelMap};

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

    #[test]
    fn test_question_to_bytes_compressed() {
        let qname = DomainName::try_from("sheets.google.com").unwrap();
        let qtype = Qtype::A;
        let qclass = ResourceRecordQClass::In;
        let question = Question::new(qname.clone(), qtype, qclass);

        let offset: u16 = 191;
        let mut label_map: LabelMap = LabelMap::new();
        // Test with no compression available
        let expected_bytes = [
            qname.to_bytes(),
            (qtype as u16).to_be_bytes().to_vec(),
            (qclass as u16).to_be_bytes().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();
        let result = question.to_bytes_compressed(offset, &mut label_map);
        assert_eq!(result.compressed_bytes, expected_bytes);
        assert_eq!(result.new_offset, (expected_bytes.len() as u16) + offset);

        // Test with "google.com" compression available
        label_map.clear();
        let google_com_labels = vec![
            DomainLabel::try_from("google").unwrap(),
            DomainLabel::try_from("com").unwrap(),
        ];
        let google_com_offset = 46;
        let offset = 97;
        label_map.insert(&google_com_labels, google_com_offset);

        let expected_bytes = [
            DomainLabel::try_from("sheets").unwrap().to_bytes(),
            create_pointer(google_com_offset).to_be_bytes().to_vec(),
            (qtype as u16).to_be_bytes().to_vec(),
            (qclass as u16).to_be_bytes().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();

        let result = question.to_bytes_compressed(offset, &mut label_map);
        assert_eq!(result.compressed_bytes, expected_bytes);
        assert_eq!(result.new_offset, (expected_bytes.len() as u16) + offset);

        // Test with ".com" compression available
        label_map.clear();
        let com_labels = vec![DomainLabel::try_from("com").unwrap()];
        let com_offset = 102;
        let offset = 55;
        label_map.insert(&com_labels, com_offset);

        let expected_bytes = [
            DomainLabel::try_from("sheets").unwrap().to_bytes(),
            DomainLabel::try_from("google").unwrap().to_bytes(),
            create_pointer(com_offset).to_be_bytes().to_vec(),
            (qtype as u16).to_be_bytes().to_vec(),
            (qclass as u16).to_be_bytes().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();

        let result = question.to_bytes_compressed(offset, &mut label_map);
        assert_eq!(result.compressed_bytes, expected_bytes);
        assert_eq!(result.new_offset, (expected_bytes.len() as u16) + offset);
    }

    #[test]
    fn test_message_questions_to_bytes_compressed() {
        let domain1 = DomainName::try_from("store.steampowered.com").unwrap();
        let domain2 = DomainName::try_from("example.com").unwrap();
        let domain3 = DomainName::try_from("twister.example.com").unwrap();
        let domain4 = DomainName::try_from("twister.hello.com").unwrap();

        let question1 = Question::new(domain1.clone(), Qtype::A, ResourceRecordQClass::In);
        let question2 = Question::new(domain2.clone(), Qtype::A, ResourceRecordQClass::In);
        let question3 = Question::new(domain3.clone(), Qtype::A, ResourceRecordQClass::In);
        let question4 = Question::new(domain4.clone(), Qtype::A, ResourceRecordQClass::In);

        let questions = MessageQuestions::new(vec![question1, question2.clone()]);
        let mut label_map = LabelMap::new();
        let offset = 0;

        let expected_bytes = [
            DomainLabel::try_from("store").unwrap().to_bytes(),
            DomainLabel::try_from("steampowered").unwrap().to_bytes(),
            // Offset at this point should be 19
            DomainLabel::try_from("com").unwrap().to_bytes(),
            vec![0],
            (Qtype::A as u16).to_be_bytes().to_vec(),
            (ResourceRecordQClass::In as u16).to_be_bytes().to_vec(),
            DomainLabel::try_from("example").unwrap().to_bytes(),
            create_pointer(19).to_be_bytes().to_vec(),
            (Qtype::A as u16).to_be_bytes().to_vec(),
            (ResourceRecordQClass::In as u16).to_be_bytes().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();

        let result = questions.to_bytes_compressed(offset, &mut label_map);
        assert_eq!(result.compressed_bytes, expected_bytes);
        assert_eq!(result.new_offset, 42);

        let questions = MessageQuestions::new(vec![question2, question3, question4]);
        let offset = 10;
        label_map.clear();
        let expected_bytes = [
            DomainLabel::try_from("example").unwrap().to_bytes(),
            // Offset at this point should be 8
            DomainLabel::try_from("com").unwrap().to_bytes(),
            vec![0],
            (Qtype::A as u16).to_be_bytes().to_vec(),
            (ResourceRecordQClass::In as u16).to_be_bytes().to_vec(),
            DomainLabel::try_from("twister").unwrap().to_bytes(),
            create_pointer(offset).to_be_bytes().to_vec(),
            (Qtype::A as u16).to_be_bytes().to_vec(),
            (ResourceRecordQClass::In as u16).to_be_bytes().to_vec(),
            DomainLabel::try_from("twister").unwrap().to_bytes(),
            DomainLabel::try_from("hello").unwrap().to_bytes(),
            create_pointer(offset + 8).to_be_bytes().to_vec(),
            (Qtype::A as u16).to_be_bytes().to_vec(),
            (ResourceRecordQClass::In as u16).to_be_bytes().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();

        let result = questions.to_bytes_compressed(offset, &mut label_map);
        assert_eq!(result.compressed_bytes, expected_bytes);
        assert_eq!(result.new_offset, 61);
    }

    #[test]
    fn test_question_parse() {
        let qname = DomainName::try_from("example.net").unwrap();
        let qtype = Qtype::A;
        let qclass = ResourceRecordQClass::In;

        let question = Question::new(qname, qtype, qclass);
        let question_bytes = question.to_bytes();

        let (parsed_question, remaining_input) = Question::parse(&question_bytes, None).unwrap();
        assert_eq!(parsed_question, question);
        assert_eq!(remaining_input.len(), 0);
    }
}
