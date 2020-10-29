use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::manifest::Source;
use super::meta::Meta;

pub type IndexMap = HashMap<String, Index>;

#[derive(Deserialize, Serialize, PartialEq)]
pub struct Index {
    /// Meta descriptor of manifest content
    pub meta: Meta,

    /// Meta content source
    pub source: Source,
}
