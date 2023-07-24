mod label;

pub use label::{DomainLabel, DomainLabelValidationError};


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
        let split = value.split(".");
        // if split.clone().count() == 0 {
        //     return Self::Error;
        // }
        let mut e: Self::Error;
        let domain_labels = split.map_while(|domain_part| {
            match DomainLabel::try_from(domain_part) {
                Ok(label) => Some(label),
                Err(e) => {

                },
            }
        })
        // let domain_labels: Vec<DomainLabel> = split.map(|domain_part| {
        //     DomainLabel::from(domain_part)
        // })
        // .collect();
    }
}