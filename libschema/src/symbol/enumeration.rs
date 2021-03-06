use serde::{Deserialize, Serialize};

use crate::symbol::Declaration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Value that are explicitly set in code expressions
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enumeration {
    #[serde(flatten)]
    pub declaration: Declaration,

    pub entries: Vec<Entry>,
}
