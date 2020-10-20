use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::meta::Meta;
use super::manifest::Source;

pub type IndexMap = HashMap<String, Index>;

#[derive(Deserialize, Serialize, PartialEq)]
pub struct Index {
    /// Meta descriptor of manifest content
    pub meta: Meta,

    /// Meta content source
    pub source: Source,
}
