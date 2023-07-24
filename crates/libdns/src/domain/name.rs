use itertools::Itertools;

use super::{DomainLabelValidationError, DomainLabel};

pub enum DomainNameError {
    LabelValidationError {
        domain_name: String,
        domain_label: String,
        validation_error: DomainLabelValidationError,
    }
}

pub struct DomainName {
    domain_labels: Vec<DomainLabel>,
    domain_name: String,
}

impl TryFrom<&str> for DomainName {
    // TODO: Create proper Error type!
    type Error = DomainNameError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let split = value.split('.');
        let mut err: Option<DomainNameError> = None;
        let domain_labels = split.map_while(|domain_part| {
            match DomainLabel::try_from(domain_part) {
                Ok(label) => Some(label),
                Err(e) => {
                    err = Some(DomainNameError::LabelValidationError { domain_name: value.to_string(), domain_label: domain_part.to_string(), validation_error: e });
                    None
                },
            }
        })
        .collect_vec();

        if let Some(e) = err {
            return Err(e);
        }

        Ok(Self { domain_labels, domain_name: value.to_string() })
    }
}