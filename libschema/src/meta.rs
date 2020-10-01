use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Meta {
    /// Name of this manifest/bundle
    pub name: String,

    /// Manifest/bundle description
    pub description: Option<String>,

    /// Manifest/bundle author
    pub author: Option<String>,
}
