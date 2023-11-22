pub mod header;
pub mod question;
pub mod resource_record;

// Placeholders
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Question = 0,
    Answer = 1,
}

impl TryFrom<u8> for MessageType {
    // Use an empty error, because it's pretty clear what's the issue if this fails
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageType::Question),
            1 => Ok(MessageType::Answer),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryOpcode {
    /// A standard query (QUERY)
    Query = 0,
    /// An inverse query (IQUERY)
    Iquery = 1,
    /// A server status request (STATUS)
    Status = 2,
    /// Numbers 3-15 are reserved for future use. In this implementation, any number greater
    /// than `3` will simply be treated as reserved, and it will not be used for any purpose
    Reserved = 3,
}

impl TryFrom<u8> for QueryOpcode {
    // Use an empty error, because it's pretty clear what's the issue if this fails
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Query),
            1 => Ok(Self::Iquery),
            2 => Ok(Self::Status),
            3..=15 => Ok(Self::Reserved),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseCode {
    /// No error condition
    NoError = 0,
    /// Format error - The name server was unable to interpret the query.
    FormatError = 1,
    /// Server failure - The name server was unable to process this query due to a
    /// problem with the name server.
    ServerFailure = 2,
    /// Name Error - Meaningful only for responses from an authoritative name
    /// server, this code signifies that the domain name referenced in the query does
    /// not exist.
    NameError = 3,
    /// Not Implemented - The name server does not support the requested kind of query.
    NotImplemented = 4,
    /// Refused - The name server refuses to perform the specified operation for
    /// policy reasons. For example, a name server may not wish to provide the
    /// information to the particular requester, or a name server may not wish to perform
    /// a particular operation (e.g., zone transfer) for particular data.
    Refused = 5,
    // Numbers 6-15 are reserved for future use. In this implementation, any number greater
    /// than `6` will simply be treated as reserved, and it will not be used for any purpose
    Reserved = 6,
}

impl TryFrom<u8> for ResponseCode {
    // Use an empty error, because it's pretty clear what's the issue if this fails
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoError),
            1 => Ok(Self::FormatError),
            2 => Ok(Self::ServerFailure),
            3 => Ok(Self::NameError),
            4 => Ok(Self::NotImplemented),
            5 => Ok(Self::Refused),
            6..=15 => Ok(Self::Reserved),
            _ => Err(()),
        }
    }
}

// pub struct DnsMessage {
//     header: Header,
//     message_type: MessageType,
// }

// pub struct DnsQuery {
//     header: Header,
//     question: Question,
//     answer: Answer,
//     authority: Authority,
//     additional: Additional,
// }

// pub struct DnsAnswer {

// }
