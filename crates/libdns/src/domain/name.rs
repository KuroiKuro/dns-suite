use std::{str::FromStr, collections::{HashMap, VecDeque}};

use ascii::{AsciiChar, AsciiString};
use itertools::Itertools;
use thiserror::Error;

use super::{DomainLabel, DomainLabelValidationError};

const DOMAIN_NAME_LENGTH_LIMIT: usize = 255;
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

        let total_label_len: usize = domain_labels.iter().map(|label| label.len()).sum();
        if total_label_len > DOMAIN_NAME_LENGTH_LIMIT {
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
    pub fn to_bytes(&self) -> Vec<u8> {
        self.domain_labels
            .iter()
            .chain(&[DomainLabel::new_empty()])
            .flat_map(|label| label.to_bytes())
            .collect_vec()
    }

    pub fn to_bytes_compressed(&self, base_offset: u16, label_map: &mut HashMap<&[DomainLabel], u16>) -> Vec<u8> {
        // Check if the entire domain name is in the hashmap
        if let Some(offset) = label_map.get(self.domain_labels.as_slice()) {
            let pointer: u16 = POINTER_PREFIX | offset;
            return pointer.to_be_bytes().to_vec();
        } else {
            label_map.insert(&self.domain_labels, base_offset);
        }

        let mut rolling_offset = base_offset;
        let mut popped_labels: Vec<DomainLabel> = Vec::with_capacity(self.domain_labels.len());
        let mut bytes: Vec<u8> = Vec::new();
        let mut labels: Vec<DomainLabel> = self.domain_labels.clone();
        labels.reverse();
        loop {
            let popped = labels.pop();
            if popped.is_none() {
                break;
            }
            let label = popped.unwrap();
            rolling_offset += label.len() as u16;
            popped_labels.push(label);
            if let Some(offset) = label_map.get(labels.as_slice()) {
                // We have found an offset we can use
                let pointer = POINTER_PREFIX | offset;
                let label_bytes: Vec<u8> = popped_labels
                    .iter()
                    .flat_map(|label| label.to_bytes())
                    .chain(pointer.to_be_bytes().into_iter())
                    .collect();
                return label_bytes;
            } else {
                // Offset was not found, so cache the remaining labels into label_map
                label_map.insert(&labels, rolling_offset);
            }
        }
        // If loop was broken, no offset was used
        labels
            .into_iter()
            .flat_map(|label| label.to_bytes())
            .collect_vec()
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
}
