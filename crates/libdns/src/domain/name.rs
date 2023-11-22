use std::str::FromStr;

use ascii::{AsciiChar, AsciiString};
use itertools::Itertools;
use thiserror::Error;

use crate::{BytesSerializable, CompressedBytesSerializable, LabelMap, ParseDataError, types::DomainPointer, POINTER_PREFIX, SerializeCompressedResult};

use super::{DomainLabel, DomainLabelValidationError};

const DOMAIN_NAME_LENGTH_LIMIT: u8 = 255;

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

#[derive(Clone, Debug)]
pub struct DomainName {
    domain_labels: Vec<DomainLabel>,
}

impl DomainName {
    pub fn labels(&self) -> &[DomainLabel] {
        &self.domain_labels
    }
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

        Ok(Self { domain_labels })
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

impl BytesSerializable for DomainName {
    fn to_bytes(&self) -> Vec<u8> {
        self.domain_labels
            .iter()
            .chain(&[DomainLabel::new_empty()])
            .flat_map(|label| label.to_bytes())
            .collect_vec()
    }

    /// Pass in a byte-serialized sequence of labels
    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), ParseDataError> {
        let mut domain_labels: Vec<DomainLabel> = Vec::new();
        let mut remaining: &[u8] = bytes;
        // while let Ok((label, remaining)) = DomainLabel::parse(bytes) {
        loop {
            let (label, r) = match DomainLabel::parse(remaining) {
                Ok(l) => l,
                // There should be no parsing error here, because we should encounter
                // the null terminating label first before parsing other data
                Err(_) => return Err(ParseDataError::InvalidByteStructure)
            };
            remaining = r;
            let is_empty = label.is_empty();
            domain_labels.push(label);
            if is_empty {
                break;
            }
        };
        Ok((Self { domain_labels }, remaining))
    }
}

impl CompressedBytesSerializable for DomainName {
    fn to_bytes_compressed(&self, base_offset: u16, label_map: &mut LabelMap) -> SerializeCompressedResult {
        // We need to check if the labels exist first before inserting into the map, otherwise we will always
        // get a domain pointer even when the labels were inserted for the first time in this function call
        let (compressed_bytes, new_offset) = {
            let result = label_map.get_domain_ptr(&self.domain_labels);
            match result {
                Some((domain_ptr, remaining_labels)) => {
                    // If there were already at least some of the labels inserted into the map, we will then have
                    // a domain pointer. Calculate the new offset based on the remaining labels and domain pointer
                    // size and serialize the bytes with the combination of the remaining, non-inserted labels and
                    // the domain pointer
                    if remaining_labels.is_empty() {
                        (domain_ptr.to_bytes(), base_offset + DomainPointer::SIZE)
                    } else {
                        let remaining_labels_offset: u16 = remaining_labels
                            .iter()
                            .map(|label| label.len_bytes() as u16)
                            .sum();
                        let bytes = remaining_labels
                            .iter()
                            .flat_map(|label| label.to_bytes())
                            .chain(domain_ptr.to_bytes())
                            .collect();
                        (bytes, base_offset + remaining_labels_offset + DomainPointer::SIZE)
                    }
                },
                None => {
                    // If not a single label exists in the map, then the output will be exactly the same as the non-compressed
                    // version
                    let bytes = self.to_bytes();
                    let new_offset = bytes.len() as u16 + base_offset;
                    (bytes, new_offset)
                }
            }
        };

        label_map.insert(&self.domain_labels, base_offset);
        SerializeCompressedResult { compressed_bytes, new_offset }
    }

    fn parse_compressed<'a>(
        bytes: &'a [u8],
        base_offset: u16,
        label_map: &mut LabelMap,
    ) -> (Result<(Self, &'a [u8]), ParseDataError>, u16)
    where
        Self: std::marker::Sized,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::create_pointer;

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
        let mut label_map = LabelMap::new();
        let offset = 19;
        let full_domain_name = DomainName::try_from("outlook.live.com").unwrap();
        let expected_bytes: Vec<u8> = vec![
            vec![7, 111, 117, 116, 108, 111, 111, 107],
            vec![4, 108, 105, 118, 101],
            vec![3, 99, 111, 109],
            vec![0],
        ]
        .into_iter()
        .flatten()
        .collect();

        let result = full_domain_name.to_bytes_compressed(offset, &mut label_map);
        let new_offset = offset + (expected_bytes.len() as u16);
        assert_eq!(expected_bytes, result.compressed_bytes);
        assert_eq!(new_offset, result.new_offset);

        // Test with full domain present in the map
        let result = full_domain_name.to_bytes_compressed(new_offset, &mut label_map);
        // With the full domain, only a single domain pointer to the original offset should be returned
        let expected_bytes = DomainPointer::new(offset).to_bytes();
        let new_offset = new_offset + DomainPointer::SIZE;
        assert_eq!(expected_bytes, result.compressed_bytes);
        assert_eq!(new_offset, result.new_offset);

        // Test with "live.com" in the label_map
        label_map.clear();
        let offset = 31;
        let partial_domain_name = DomainName::try_from("live.com").unwrap();
        let inserted_labels = partial_domain_name.labels();
        label_map.insert(inserted_labels, offset);
        // We should have the uncompressed bytes for "live" and then the domain pointer to "com"
        let expected_bytes: Vec<u8> = vec![
            vec![7, 111, 117, 116, 108, 111, 111, 107],
            create_pointer(offset).to_be_bytes().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect();
        let result = full_domain_name.to_bytes_compressed(offset, &mut label_map);
        let new_offset = offset + (expected_bytes.len() as u16);
        assert_eq!(expected_bytes, result.compressed_bytes);
        assert_eq!(new_offset, result.new_offset);

        // Test with "com" in the label_map
        label_map.clear();
        let offset = 47;
        let com_labels = vec![DomainLabel::try_from("com").unwrap()];
        label_map.insert(&com_labels, offset);
        // We should have the uncompressed bytes for "outlook" and "live" and then the domain pointer to "com"
        let expected_bytes: Vec<u8> = vec![
            vec![7, 111, 117, 116, 108, 111, 111, 107],
            vec![4, 108, 105, 118, 101],
            create_pointer(offset).to_be_bytes().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect();
        let result = full_domain_name.to_bytes_compressed(offset, &mut label_map);
        let new_offset = offset + (expected_bytes.len() as u16);
        assert_eq!(expected_bytes, result.compressed_bytes);
        assert_eq!(new_offset, result.new_offset);
    }

    #[test]
    fn test_domain_name_parse() {
        let bytes = [
            4,
            AsciiChar::d as u8,
            AsciiChar::o as u8,
            AsciiChar::c as u8,
            AsciiChar::s as u8,
            9,
            AsciiChar::r as u8,
            AsciiChar::u as u8,
            AsciiChar::s as u8,
            AsciiChar::t as u8,
            AsciiChar::Minus as u8,
            AsciiChar::l as u8,
            AsciiChar::a as u8,
            AsciiChar::n as u8,
            AsciiChar::g as u8,
            3,
            AsciiChar::o as u8,
            AsciiChar::r as u8,
            AsciiChar::g as u8,
            0
        ];

        let (domain_name, remaining) = DomainName::parse(&bytes).unwrap();
        // 3 + 1 because of the null terminating label
        assert_eq!(domain_name.domain_labels.len(), 4);
        assert_eq!(remaining.len(), 0);

        // Test without null terminator
        let bytes = [
            5,
            AsciiChar::e as u8,
            AsciiChar::r as u8,
            AsciiChar::r as u8,
            AsciiChar::o as u8,
            AsciiChar::r as u8,
            3,
            AsciiChar::o as u8,
            AsciiChar::r as u8,
            AsciiChar::g as u8,
        ];

        let result = DomainName::parse(&bytes);
        assert!(result.is_err());
    }
}
