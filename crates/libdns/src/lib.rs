use std::collections::{HashMap, VecDeque, hash_map::Entry};

use domain::DomainLabel;
use thiserror::Error;
use types::DomainPointer;

pub mod domain;
pub mod message;
pub mod parse_utils;
pub mod rr;
pub mod types;

type MessageOffset = u16;

// pub type LabelMap = HashMap<VecDeque<DomainLabel>, u16>;
// All pointers must have `11` as the first two bits
pub const POINTER_PREFIX: u16 = 0xC000;

pub struct LabelMapInsertResult {
    pub inserted_records: usize,
    pub new_offset: MessageOffset,
    pub remaining_labels: Vec<DomainLabel>,
}

pub struct LabelMap {
    label_to_offset_map: HashMap<Vec<DomainLabel>, MessageOffset>,
    offset_to_label_map: HashMap<MessageOffset, Vec<DomainLabel>>,
}

impl LabelMap {
    pub fn new() -> Self {
        Self { label_to_offset_map: HashMap::new(), offset_to_label_map: HashMap::new() }
    }

    /// Gets a domain pointer from the label map if it exists. If a domain pointer exists,
    /// then a vec of the remaining labels is also returned. This is mainly used when
    /// performing serialization of compressed messages.
    /// 
    /// For example, if we have a set of labels ["help", "example", "com"], and we found
    /// a pointer for ["example", "com"], then we will return the domain pointer for
    /// ["example", "com"], as well as the remaining labels ["help"].
    pub fn get_domain_ptr(&self, labels: &[DomainLabel]) -> Option<(DomainPointer, Vec<DomainLabel>)> {
        let mut labels_check = labels.to_vec();
        let mut remaining_labels = Vec::new();
        while !labels_check.is_empty() {
            if let Some(offset) = self.label_to_offset_map.get(&labels_check) {
                return Some((DomainPointer::new(*offset), remaining_labels));
            }
            remaining_labels.push(labels_check.pop().unwrap());
        }
        None
    }

    pub fn get_offset(&self, labels: &[DomainLabel]) -> Option<&MessageOffset> {
        self.label_to_offset_map.get(&labels.to_vec())
    }

    /// Insert a set of domain labels into the map. This will continuously insert sets
    /// of labels while there are either labels remaining in the overall set, or until
    /// we encounter a set of labels that has already been inserted into the map, at
    /// which point we will stop the insertion, as it means that all of the subsequent
    /// label sets have already been inserted in a prior `insert` operation
    pub fn insert(&mut self, domain_labels: &[DomainLabel], offset: MessageOffset) -> LabelMapInsertResult {
        let mut inserted_records = 0;
        let mut current_offset = offset;
        // Use slicing of the domain_labels vec to iterate through the labels. With slicing, we can
        // increase the reference index of the first accessor, to incrementally exclude labels starting
        // from the front
        let max_index = domain_labels.len() - 1;
        let mut remaining_labels: Vec<DomainLabel> = Vec::new();
        for i in 0..=max_index {
            let current_label_set = domain_labels[i..].to_vec();
            match self.label_to_offset_map.entry(current_label_set) {
                // If the entry is occupied, it means that the set of labels, as well
                // as subsequent sets have already been inserted into the map
                Entry::Occupied(entry) => {
                    remaining_labels = entry.key().to_vec();
                    break;
                },
                Entry::Vacant(entry) => {
                    entry.insert(current_offset);
                    inserted_records += 1;
                    current_offset += domain_labels[i].len() as u16;
                },
            }
        }
        LabelMapInsertResult { inserted_records, new_offset: current_offset, remaining_labels }
    }

    pub fn clear(&mut self) {
        self.label_to_offset_map.clear();
        self.offset_to_label_map.clear();
    }
}

impl Default for LabelMap {
    fn default() -> Self {
        Self::new()
    }
}

/// A generic error enum used when parsing of a certain item from its byte-serialized
/// data fails. The intention of this is to allow for easier error propagation using
/// the `?` operator. Use of the tracing
#[derive(Debug, Error)]
pub enum ParseDataError {
    #[error("Invalid byte structure")]
    InvalidByteStructure,
    #[error("No data to parse")]
    EmptyData,
    #[error("Invalid domain pointer in compressed message")]
    InvalidDomainPointer,
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

#[cfg(test)]
mod tests {
    use crate::domain::DomainName;

    use super::*;

    #[test]
    fn test_label_map_insert() {
        // Test with insertion of labels into empty map
        let mut label_map = LabelMap::new();
        let labels = vec![
            DomainLabel::try_from("api").unwrap(),
            DomainLabel::try_from("stripe").unwrap(),
            DomainLabel::try_from("com").unwrap(),
        ];
        let result = label_map.insert(&labels, 0);
        assert_eq!(result.inserted_records, 3);
        assert_eq!(result.new_offset, labels.iter().map(|label| label.len() as u16).sum::<u16>());
        assert_eq!(result.remaining_labels, Vec::new());

        let offset = result.new_offset;
        // Test that no records will be inserted if they already exist
        let result = label_map.insert(&labels, offset);
        assert_eq!(result.inserted_records, 0);
        assert_eq!(result.new_offset, offset);
        assert_eq!(result.remaining_labels, labels);

        // Test partial label insertion
        let labels = vec![
            DomainLabel::try_from("docs").unwrap(),
            DomainLabel::try_from("stripe").unwrap(),
            DomainLabel::try_from("com").unwrap(),
        ];

        let result = label_map.insert(&labels, offset);
        assert_eq!(result.inserted_records, 1);
        assert_eq!(result.new_offset, offset + labels[0].len() as u16);
        assert_eq!(result.remaining_labels, labels[1..].to_vec());
    }
}
