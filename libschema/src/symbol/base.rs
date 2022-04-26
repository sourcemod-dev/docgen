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
    fn eq(&self, other: &Self) -> bool {
        // Only compare against the docs
        self.docs == other.docs
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

impl Metable for Declaration {
    fn metadata(&mut self) -> &mut Option<Metadata> {
        &mut self.documentation.metadata
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DocLocation(i64);

impl From<DocLocation> for usize {
    fn from(v: DocLocation) -> Self {
        v.0 as usize
    }
}

impl From<usize> for DocLocation {
    fn from(v: usize) -> Self {
        DocLocation(v as i64)
    }
}

pub trait Metable {
    fn metadata(&mut self) -> &mut Option<Metadata>;
}
