use std::{str::FromStr, collections::{HashMap, VecDeque}};

use ascii::{AsciiChar, AsciiString};
use itertools::Itertools;
use thiserror::Error;

use crate::{LabelMap, BytesSerializable, CompressedBytesSerializable};

use super::{DomainLabel, DomainLabelValidationError};

const DOMAIN_NAME_LENGTH_LIMIT: u8 = 255;
// All pointers must have `11` as the first two bits
const POINTER_PREFIX: u16 = 0xC000;

#[derive(Debug, Error)]
pub enum DomainNameValidationError {
    #[error("Validation error on '{domain_label}' of '{domain_name} due to {validation_error}")]
    LabelValidationError {
        domain_name: String,
        domain_label: String,
        validation_error: DomainLabelValidationError,
    },
    #[error("Domain Name ('{0}') is too long (max: {1})")]
    NameTooLong(String, usize),
    #[error("Domain Name contains invalid ASCII ('{0}')")]
    InvalidAscii(String),
}

pub struct DomainName {
    domain_labels: Vec<DomainLabel>,
    domain_name: AsciiString,
}

impl TryFrom<&str> for DomainName {
    type Error = DomainNameValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let ascii_str = match AsciiString::from_str(value) {
            Ok(s) => s,
            Err(_) => return Err(DomainNameValidationError::InvalidAscii(value.to_string())),
        };
        let split = ascii_str.split(AsciiChar::Dot);
        let mut err: Option<DomainNameValidationError> = None;
        let domain_labels = split
            .map_while(|domain_part| match DomainLabel::try_from(domain_part) {
                Ok(label) => Some(label),
                Err(e) => {
                    err = Some(DomainNameValidationError::LabelValidationError {
                        domain_name: value.to_string(),
                        domain_label: domain_part.to_string(),
                        validation_error: e,
                    });
                    None
                }
            })
            .collect_vec();

        if let Some(e) = err {
            return Err(e);
        }

        let total_label_len: usize = domain_labels.iter().map(|label| label.len() as usize).sum();
        if total_label_len > DOMAIN_NAME_LENGTH_LIMIT.into() {
            return Err(DomainNameValidationError::NameTooLong(
                value.to_string(),
                total_label_len,
            ));
        }

        Ok(Self {
            domain_labels,
            domain_name: ascii_str,
        })
    }
}

impl PartialEq for DomainName {
    fn eq(&self, other: &Self) -> bool {
        let other_labels = other.domain_labels.iter();
        self.domain_labels
            .iter()
            .zip(other_labels)
            .map(|(self_label, other_label)| self_label == other_label)
            .all_equal()
    }
}

impl DomainName {
    
}

impl BytesSerializable for DomainName {
    type ParseError = ();
    
    fn to_bytes(&self) -> Vec<u8> {
        self.domain_labels
            .iter()
            .chain(&[DomainLabel::new_empty()])
            .flat_map(|label| label.to_bytes())
            .collect_vec()
    }

    fn parse(bytes: &[u8]) -> Result<Self, Self::ParseError> {
        todo!()
    }
}

impl CompressedBytesSerializable for DomainName {
    type ParseError = ();

    fn to_bytes_compressed(&self, base_offset: u16, label_map: &mut LabelMap) -> (Vec<u8>, u16) {
        // Check if the entire domain name is in the hashmap
        let domain_labels_vec_deque = VecDeque::from(self.domain_labels.clone());
        if let Some(offset) = label_map.get(&domain_labels_vec_deque) {
            let pointer: u16 = POINTER_PREFIX | offset;
            // A pointer is 2 octets
            let new_offset = base_offset + 2;
            return (pointer.to_be_bytes().to_vec(), new_offset);
        } else {
            label_map.insert(domain_labels_vec_deque, base_offset);
        }

        let mut rolling_offset = base_offset;
        let mut popped_labels: Vec<DomainLabel> = Vec::with_capacity(self.domain_labels.len());
        let mut labels: VecDeque<DomainLabel> = VecDeque::from(self.domain_labels.clone());
        loop {
            let popped = labels.pop_front();
            if popped.is_none() {
                break;
            }
            let label = popped.unwrap();
            rolling_offset += label.bytes_len() as u16;
            popped_labels.push(label);
            if let Some(offset) = label_map.get(&labels) {
                // We have found an offset we can use
                let pointer = POINTER_PREFIX | offset;
                let label_bytes: Vec<u8> = popped_labels
                    .iter()
                    .flat_map(|label| label.to_bytes())
                    .chain(pointer.to_be_bytes())
                    .collect();
                let new_offset = label_bytes.len() as u16 + base_offset;
                return (label_bytes, new_offset);
            } else {
                // Offset was not found, so cache the remaining labels into label_map
                label_map.insert(labels.clone(), rolling_offset);
            }
        }
        // If loop was broken, no offset was used
        let bytes = popped_labels
            .into_iter()
            .flat_map(|label| label.to_bytes())
            .chain([0])
            .collect_vec();
        let new_offset = (bytes.len() as u16) + base_offset;
        (bytes, new_offset)
    }

    fn parse_compressed(bytes: &[u8], base_offset: u16, label_map: &mut LabelMap) -> (Result<Self, Self::ParseError>, u16) where Self: std::marker::Sized {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_bytes() {
        let domain_name = DomainName::try_from("outlook.live.com").unwrap();
        let expected_tags: Vec<u8> = vec![
            vec![7, 111, 117, 116, 108, 111, 111, 107],
            vec![4, 108, 105, 118, 101],
            vec![3, 99, 111, 109],
            vec![0],
        ]
        .into_iter()
        .flatten()
        .collect();

        let domain_bytes = domain_name.to_bytes();
        assert_eq!(expected_tags, domain_bytes);
    }

    #[test]
    fn test_to_bytes_compressed() {
        // Test that output will be same as non-compressed version with empty hashmap
        let mut label_map = HashMap::new();
        let offset = 0;
        let domain_name = DomainName::try_from("outlook.live.com").unwrap();
        let expected_bytes: Vec<u8> = vec![
            vec![7, 111, 117, 116, 108, 111, 111, 107],
            vec![4, 108, 105, 118, 101],
            vec![3, 99, 111, 109],
            vec![0],
        ]
        .into_iter()
        .flatten()
        .collect();

        let (domain_bytes, new_offset) = domain_name.to_bytes_compressed(offset, &mut label_map);
        assert_eq!(expected_bytes, domain_bytes);
        assert_eq!(new_offset, offset + (expected_bytes.len() as u16));

        // Test with full domain present in the map
        let full_domain_labels = VecDeque::from(domain_name.domain_labels.clone());
        let full_domain_offset = 14;
        label_map.insert(full_domain_labels, full_domain_offset);
        let expected_bytes = (POINTER_PREFIX | full_domain_offset)
            .to_be_bytes()
            .to_vec();
        let (domain_bytes, new_offset) = domain_name.to_bytes_compressed(full_domain_offset, &mut label_map);
        assert_eq!(expected_bytes, domain_bytes);
        assert_eq!(new_offset, full_domain_offset + (expected_bytes.len() as u16));

        // Test with "live.com" in the label_map
        label_map.clear();
        let half_domain = DomainName::try_from("live.com").unwrap();
        let half_labels = VecDeque::from(half_domain.domain_labels.clone());
        let half_domain_offset = 19;
        label_map.insert(half_labels, half_domain_offset);
        let expected_bytes: Vec<u8> = vec![
            vec![7, 111, 117, 116, 108, 111, 111, 107],
            (POINTER_PREFIX | half_domain_offset).to_be_bytes().to_vec()
        ]
        .into_iter()
        .flatten()
        .collect();
        let (domain_bytes, new_offset) = domain_name.to_bytes_compressed(half_domain_offset, &mut label_map);
        assert_eq!(expected_bytes, domain_bytes);
        assert_eq!(new_offset, half_domain_offset + (expected_bytes.len() as u16));

        // Test with "com" in the label_map
        label_map.clear();
        let com_labels = VecDeque::from([DomainLabel::try_from("com").unwrap()]);
        let com_offset = 33;
        label_map.insert(com_labels, com_offset);
        let expected_bytes: Vec<u8> = vec![
            vec![7, 111, 117, 116, 108, 111, 111, 107],
            vec![4, 108, 105, 118, 101],
            (POINTER_PREFIX | com_offset).to_be_bytes().to_vec()
        ]
            .into_iter()
            .flatten()
            .collect();
        let (domain_bytes, new_offset) = domain_name.to_bytes_compressed(com_offset, &mut label_map);
        assert_eq!(expected_bytes, domain_bytes);
        assert_eq!(new_offset, com_offset + (expected_bytes.len() as u16));
    }
}
