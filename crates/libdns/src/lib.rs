use std::collections::{HashMap, VecDeque};

use domain::DomainLabel;

pub mod domain;
pub mod message;
pub mod rr;
pub mod types;
pub mod parse_utils;

pub type LabelMap = HashMap<VecDeque<DomainLabel>, u16>;

/// A trait for types that can serialize and parse their data with bytes
pub trait BytesSerializable {
    type ParseError;
    fn to_bytes(&self) -> Vec<u8>;
    fn parse(bytes: &[u8]) -> Result<Self, Self::ParseError> where Self: std::marker::Sized;
}

/// A trait for types that can serialize and parse their data in bytes that are
/// compressed in the specification in RFC 1035.
pub trait CompressedBytesSerializable {
    type ParseError;
    fn to_bytes_compressed(&self, base_offset: u16, label_map: &mut LabelMap) -> (Vec<u8>, u16);
    fn parse_compressed(bytes: &[u8], base_offset: u16, label_map: &mut LabelMap) -> (Result<Self, Self::ParseError>, u16) where Self: std::marker::Sized;
}
