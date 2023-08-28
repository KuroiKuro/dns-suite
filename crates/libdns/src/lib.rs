use std::collections::{HashMap, VecDeque};

use domain::DomainLabel;

pub mod domain;
pub mod message;
pub mod rr;
pub mod types;
pub mod parse_utils;

pub type LabelMap = HashMap<VecDeque<DomainLabel>, u16>;
