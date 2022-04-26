use std::ops::ShlAssign;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::metadata::Metadata;
use crate::symbol::{Declaration, Metable};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Value that are explicitly set in code expressions
    pub value: Option<String>,
}

impl Metable for Entry {
    fn metadata(&mut self) -> &mut Option<Metadata> {
        &mut self.declaration.documentation.metadata
    }
}

impl ShlAssign for Entry {
    fn shl_assign(&mut self, rhs: Self) {
        self.declaration <<= rhs.declaration;
        self.value = rhs.value;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enumeration {
    #[serde(flatten)]
    pub declaration: Declaration,

    pub entries: HashMap<String, Entry>,
}

impl Metable for Enumeration {
    fn metadata(&mut self) -> &mut Option<Metadata> {
        &mut self.declaration.documentation.metadata
    }
}

impl ShlAssign for Enumeration {
    fn shl_assign(&mut self, rhs: Self) {
        self.declaration <<= rhs.declaration;
        self.entries = rhs.entries;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DPEnumeration {
    #[serde(flatten)]
    pub declaration: Declaration,

    pub entries: Vec<Entry>,
}
