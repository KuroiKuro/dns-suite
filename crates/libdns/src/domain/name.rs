use itertools::Itertools;

use super::{DomainLabel, DomainLabelValidationError};

const DOMAIN_NAME_LENGTH_LIMIT: usize = 255;

pub enum DomainNameError {
    LabelValidationError {
        domain_name: String,
        domain_label: String,
        validation_error: DomainLabelValidationError,
    },
    NameTooLong(String, usize),
}

pub struct DomainName {
    domain_labels: Vec<DomainLabel>,
    domain_name: String,
}

impl TryFrom<&str> for DomainName {
    type Error = DomainNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let split = value.split('.');
        let mut err: Option<DomainNameError> = None;
        let domain_labels = split
            .map_while(|domain_part| match DomainLabel::try_from(domain_part) {
                Ok(label) => Some(label),
                Err(e) => {
                    err = Some(DomainNameError::LabelValidationError {
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
            return Err(DomainNameError::NameTooLong(
                value.to_string(),
                total_label_len,
            ));
        }

        Ok(Self {
            domain_labels,
            domain_name: value.to_string(),
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
