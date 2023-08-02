pub trait Message {
    fn to_bytes(&self) -> Vec<u8>;
}

// Placeholders
pub enum MessageType {
    Question = 0,
    Answer = 1,
}

pub enum QueryOpcode {
    Query = 0,
    Iquery = 1,
    Status = 2,
    Reserved = 3,
}

impl From<u8> for QueryOpcode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Query,
            1 => Self::Iquery,
            2 => Self::Status,
            _ => Self::Reserved,
        }
    }
}

pub enum ResponseCode {
    NoError = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5,
    Reserved = 6,
}

struct Header {
    id: u16,
    qr: MessageType,
    opcode: QueryOpcode,
    authoritative_ans: bool,
    truncation: bool,
    recursion_desired: bool,
    recursion_available: bool,
    response_code: ResponseCode,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

pub struct DnsMessage {
    header: Header,
    message_type: MessageType,
}

pub struct DnsQuery {
    header: Header,
    question: Question,
    answer: Answer,
    authority: Authority,
    additional: Additional,
}

pub struct DnsAnswer {

}
