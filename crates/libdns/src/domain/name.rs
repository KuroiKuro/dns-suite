use std::str::FromStr;

use ascii::{AsciiChar, AsciiString};
use itertools::Itertools;
use thiserror::Error;

use crate::{
    types::DomainPointer, BytesSerializable, CompressedBytesSerializable, LabelMap, ParseDataError,
    SerializeCompressedResult, MessageOffset,
};

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
    pub fn new(labels: Vec<DomainLabel>) -> Self {
        Self { domain_labels: labels }
    }

    pub fn labels(&self) -> &[DomainLabel] {
        &self.domain_labels
    }

    pub fn from_label(labels: Vec<DomainLabel>) -> Self {
        Self { domain_labels: labels }
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
        loop {
            let (label, r) = match DomainLabel::parse(remaining) {
                Ok(l) => l,
                // There should be no parsing error here, because we should encounter
                // the null terminating label first before parsing other data
                Err(_) => return Err(ParseDataError::InvalidByteStructure),
            };
            remaining = r;
            let is_empty = label.is_empty();
            domain_labels.push(label);
            if is_empty {
                break;
            }
        }
        Ok((Self { domain_labels }, remaining))
    }
}

impl CompressedBytesSerializable for DomainName {
    fn to_bytes_compressed(
        &self,
        base_offset: u16,
        label_map: &mut LabelMap,
    ) -> SerializeCompressedResult {
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
                        (
                            bytes,
                            base_offset + remaining_labels_offset + DomainPointer::SIZE,
                        )
                    }
                }
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
        SerializeCompressedResult {
            compressed_bytes,
            new_offset,
        }
    }

    fn parse_compressed(
        full_message_bytes: &[u8],
        base_offset: MessageOffset,
    ) -> Result<(Self, MessageOffset), ParseDataError>
    where
        Self: std::marker::Sized,
    {
        // Continuously try to parse domain labels from the given bytes. Whenever a domain label cannot
        // be parsed, we will try to parse a domain pointer to use for a lookup on the label map. If the
        // lookup cannot be found, there is an error with parsing it so we return an `Err`, otherwise we
        // will combine the parsed labels with the labels in the lookup.
        // 
        // If there are no domain pointers, the method will work exactly the same as `to_bytes`
        let mut domain_labels: Vec<DomainLabel> = Vec::new();
        let mut new_offset = base_offset;
        loop {

            let bytes_to_parse = &full_message_bytes[(new_offset as usize)..];

            if let Ok((ptr, _)) = DomainPointer::parse(bytes_to_parse) {

                let ptr_location = &full_message_bytes[(ptr.offset() as usize)..];

                // Should not have an error here, if there is then the pointer is pointing to an invalid location
                match DomainName::parse(ptr_location) {
                    Ok((domain, _)) => domain_labels.extend_from_slice(domain.labels()),
                    Err(_) => return Err(ParseDataError::InvalidDomainPointer),
                };
                
                // After parsing the labels from the pointer, the domain parsing is completed so we
                // can return early
                let domain_name = DomainName::new(domain_labels);
                new_offset += DomainPointer::SIZE;
                return Ok((domain_name, new_offset));

            } else {

                // If it is a domain label instead of pointer, then we continue processing normally
                let (domain_label, _) = match DomainLabel::parse(bytes_to_parse) {
                    Ok(d) => d,
                    _ => return Err(ParseDataError::InvalidByteStructure),
                };

                new_offset += domain_label.len_bytes() as u16;
                // The last label has been parsed if it is an empty label, so we will need to break
                let is_final_label = domain_label.is_empty();
                domain_labels.push(domain_label);

                if is_final_label {
                    break;
                }
            }
        }

        Ok((DomainName::from_label(domain_labels), new_offset))
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
            0,
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

    #[test]
    fn test_domain_name_parse_compressed() {
        // Create a compressed domain name
        let mut label_map = LabelMap::new();
        let offset = 0;
        let original_domain = DomainName::try_from("chat.openai.com").unwrap();
        let outcome = original_domain.to_bytes_compressed(offset, &mut label_map);
        let compressed_message = outcome.compressed_bytes;

        let (parsed_domain, new_offset) = DomainName::parse_compressed(&compressed_message, offset).unwrap();
        assert_eq!(original_domain, parsed_domain);
        assert_eq!(outcome.new_offset, new_offset);
    }
}
