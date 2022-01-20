use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct Metadata {
    /// SVN version this symbol was introduced
    pub created: Option<Versioning>,

    /// SVN version this symbol was last modified
    pub last_updated: Option<Versioning>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Versioning {
    pub hash: String,

    /// Rev-list count
    /// Mainly used for core where product.version will be within spec paths
    pub count: u64,

    pub time: i64,
}

// Skip versioning when comparing
impl PartialEq for Versioning {
    fn eq(&self, _other: &Self) -> bool {
        self.hash == self.hash
    }
}

impl PartialOrd for Versioning {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.count.cmp(&other.count))
    }
}
