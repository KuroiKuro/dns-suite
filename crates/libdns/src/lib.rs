use std::collections::{hash_map::Entry, HashMap};

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

pub fn create_pointer(offset: u16) -> u16 {
    POINTER_PREFIX | offset
}

pub struct LabelMapInsertOutcome {
    /// The number of records (domain label sets) that were inserted into the map
    pub inserted_records: usize,
    /// The new offset after the insertion of the records
    pub new_offset: MessageOffset,
    /// The remaining labels that were not inserted into the map, because they
    /// already exist in the map
    pub remaining_labels: Vec<DomainLabel>,
}

pub struct LabelMap {
    label_to_offset_map: HashMap<Vec<DomainLabel>, MessageOffset>,
}

impl LabelMap {
    pub fn new() -> Self {
        Self {
            label_to_offset_map: HashMap::new(),
        }
    }

    /// Gets a domain pointer from the label map if it exists. If a domain pointer exists,
    /// then a vec of the remaining labels is also returned. This is mainly used when
    /// performing serialization of compressed messages.
    ///
    /// For example, if we have a set of labels ["help", "example", "com"], and we found
    /// a pointer for ["example", "com"], then we will return the domain pointer for
    /// ["example", "com"], as well as the remaining labels ["help"].
    pub fn get_domain_ptr(
        &self,
        labels: &[DomainLabel],
    ) -> Option<(DomainPointer, Vec<DomainLabel>)> {
        let max_idx = labels.len() - 1;
        let mut remaining_labels = Vec::new();
        for i in 0..=max_idx {
            let current_label_set = &labels[i..];
            if let Some(offset) = self.label_to_offset_map.get(current_label_set) {
                return Some((DomainPointer::new(*offset), remaining_labels));
            } else {
                remaining_labels.push(labels[i].clone());
            }
        }
        None
    }

    /// Get the offset from a set of labels
    pub fn get_offset(&self, labels: &[DomainLabel]) -> Option<&MessageOffset> {
        self.label_to_offset_map.get(&labels.to_vec())
    }

    /// Insert a set of domain labels into the map. This will continuously insert sets
    /// of labels while there are either labels remaining in the overall set, or until
    /// we encounter a set of labels that has already been inserted into the map, at
    /// which point we will stop the insertion, as it means that all of the subsequent
    /// label sets have already been inserted in a prior `insert` operation. If the
    /// first label set has been already inserted into the map before, then calling
    /// method will not cause any insertion to occur.
    pub fn insert(
        &mut self,
        domain_labels: &[DomainLabel],
        offset: MessageOffset,
    ) -> LabelMapInsertOutcome {
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
                }
                Entry::Vacant(entry) => {
                    // self.offset_to_label_map.insert(current_offset, entry.key().clone());
                    entry.insert(current_offset);
                    inserted_records += 1;
                    current_offset += domain_labels[i].len_bytes() as u16;
                }
            }
        }
        LabelMapInsertOutcome {
            inserted_records,
            new_offset: current_offset,
            remaining_labels,
        }
    }

    pub fn clear(&mut self) {
        self.label_to_offset_map.clear();
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

/// The return result type of the `to_bytes_compressed` method of the
/// `CompressedBytesSerializable` trait
pub struct SerializeCompressedOutcome {
    compressed_bytes: Vec<u8>,
    new_offset: MessageOffset,
}

/// A trait for types that can serialize and parse their data in bytes that are
/// compressed in the specification in RFC 1035.
pub trait CompressedBytesSerializable {
    /// Serialize the data into bytes, following the compression rules in RFC 1035.
    /// Using this method is just an indication that you would like the data to be
    /// compressed if necessary, if no compression is possible then the output of
    /// this method will be as if you called the regular `to_bytes` method defined
    /// in the `BytesSerializable` trait.
    fn to_bytes_compressed(
        &self,
        base_offset: u16,
        label_map: &mut LabelMap,
    ) -> SerializeCompressedOutcome;

    /// Parse data that has been compressed. For this to work, we need to have the
    /// full message retrieved from the socket and loaded into memory. All calls
    /// to this method will then access the same message in memory, which is safe
    /// as parsing operations are read only. This access pattern is advantageous
    /// because it avoids unnecessary copies.
    fn parse_compressed<'a>(
        full_message_bytes: &'a [u8],
        current_offset: MessageOffset,
    ) -> Result<(Self, MessageOffset), ParseDataError>
    where
        Self: std::marker::Sized;
}

#[cfg(test)]
mod tests {
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
        assert_eq!(
            result.new_offset,
            labels
                .iter()
                .map(|label| label.len_bytes() as u16)
                .sum::<u16>()
        );
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
        assert_eq!(result.new_offset, offset + labels[0].len_bytes() as u16);
        assert_eq!(result.remaining_labels, labels[1..].to_vec());
    }

    #[test]
    fn test_get_domain_ptr() {
        let mut label_map = LabelMap::new();
        let labels = vec![
            DomainLabel::try_from("api").unwrap(),
            DomainLabel::try_from("stripe").unwrap(),
            DomainLabel::try_from("com").unwrap(),
        ];

        let offset = 29;
        label_map.insert(&labels, offset);
        let (domain_ptr, remaining_labels) = label_map.get_domain_ptr(&labels).unwrap();
        let expected_ptr = DomainPointer::new(offset);
        assert_eq!(domain_ptr.to_bytes(), expected_ptr.to_bytes());
        assert!(remaining_labels.is_empty());

        // Test subdomain
        let partial_labels = vec![
            DomainLabel::try_from("stripe").unwrap(),
            DomainLabel::try_from("com").unwrap(),
        ];
        let (domain_ptr, remaining_labels) = label_map.get_domain_ptr(&partial_labels).unwrap();
        let expected_offset = offset + labels[0].len_bytes() as u16;
        let expected_ptr = DomainPointer::new(expected_offset);
        assert_eq!(domain_ptr.to_bytes(), expected_ptr.to_bytes());
        assert_eq!(remaining_labels, vec![]);

        // Test subdomain with remaining labels
        let partial_labels = vec![
            DomainLabel::try_from("docs").unwrap(),
            DomainLabel::try_from("stripe").unwrap(),
            DomainLabel::try_from("com").unwrap(),
        ];
        let (domain_ptr, remaining_labels) = label_map.get_domain_ptr(&partial_labels).unwrap();
        let expected_offset = offset + labels[0].len_bytes() as u16;
        let expected_ptr = DomainPointer::new(expected_offset);
        assert_eq!(domain_ptr.to_bytes(), expected_ptr.to_bytes());
        assert_eq!(remaining_labels, vec![partial_labels[0].clone()]);
    }
}
