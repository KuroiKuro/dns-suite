use std::collections::{HashMap, VecDeque};

use domain::DomainLabel;
use thiserror::Error;

pub mod domain;
pub mod message;
pub mod parse_utils;
pub mod rr;
pub mod types;

pub type LabelMap = HashMap<VecDeque<DomainLabel>, u16>;

/// A generic error enum used when parsing of a certain item from its byte-serialized
/// data fails. The intention of this is to allow for easier error propagation using
/// the `?` operator. Use of the tracing
#[derive(Debug, Error)]
pub enum ParseDataError {
    #[error("Invalid byte structure")]
    InvalidByteStructure,
    #[error("No data to parse")]
    EmptyData,
}

/// A trait for types that can serialize and parse their data with bytes
pub trait BytesSerializable {
    fn to_bytes(&self) -> Vec<u8>;
    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), ParseDataError>
    where
        Self: std::marker::Sized;
}

/// A trait for types that can serialize and parse their data in bytes that are
/// compressed in the specification in RFC 1035.
pub trait CompressedBytesSerializable {
    fn to_bytes_compressed(&self, base_offset: u16, label_map: &mut LabelMap) -> (Vec<u8>, u16);
    fn parse_compressed<'a>(
        bytes: &'a [u8],
        base_offset: u16,
        label_map: &mut LabelMap,
    ) -> (Result<(Self, &'a [u8]), ParseDataError>, u16)
    where
        Self: std::marker::Sized;
}
