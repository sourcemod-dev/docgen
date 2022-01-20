use std::ops::ShlAssign;

use serde::{Deserialize, Serialize};
use spdcp::Comment;

use crate::metadata::Metadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Documentation {
    #[serde(default)]
    pub ref_line: u64,

    #[serde(default)]
    /// Documentation starting byte
    pub doc_start: DocLocation,

    #[serde(default)]
    /// Documentation ending byte
    pub doc_end: DocLocation,

    /// Parsed documentation
    pub docs: Option<Comment>,

    pub metadata: Option<Metadata>,
}

impl ShlAssign for Documentation {
    fn shl_assign(&mut self, rhs: Self) {
        self.ref_line = rhs.ref_line;
        self.doc_start = rhs.doc_start;
        self.doc_end = rhs.doc_end;
        self.docs = rhs.docs;
        // Don't update metadata
    }
}

impl PartialEq for Documentation {
    fn eq(&self, _other: &Self) -> bool {
        // Ignore difference, since we don't care about the documentation changes
        self.docs == self.docs
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Base symbol declaration
pub struct Declaration {
    pub name: String,

    #[serde(flatten)]
    pub documentation: Documentation,
}

impl ShlAssign for Declaration {
    fn shl_assign(&mut self, rhs: Self) {
        self.name = rhs.name;
        self.documentation <<= rhs.documentation;
    }
}

impl Declaration {
    pub fn metadata(&mut self) -> &mut Option<Metadata> {
        &mut self.documentation.metadata
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DocLocation(i64);

impl Default for DocLocation {
    fn default() -> Self {
        Self(0)
    }
}

impl From<usize> for DocLocation {
    fn from(v: usize) -> Self {
        Self(v as i64)
    }
}

impl Into<usize> for DocLocation {
    fn into(self) -> usize {
        self.0 as usize
    }
}
