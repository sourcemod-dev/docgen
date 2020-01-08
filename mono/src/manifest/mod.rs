use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manifest on remote/local includes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeManifest {
    /// List of include pair (include key, fetch endpoint)
    pub includes: HashMap<String, String>,

    /// Timestamp in which it was created/generated, useful for cache busting
    pub timestamp: u64,
}
