use serde::{Deserialize, Serialize};
use spdcp::Comment;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Base symbol declaration
pub struct Declaration {
    pub name: String,

    #[serde(flatten)]
    pub documentation: Documentation,
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
