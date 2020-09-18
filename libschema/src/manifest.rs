use serde::Deserialize;

use super::meta::Meta;

#[derive(Deserialize)]
pub struct Manifest {
    /// Meta descriptor of manifest content
    pub meta: Meta,

    pub source: Source,

    /// Used as regex glob pattern when Git is selected
    pub patterns: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub enum SourceType {
    /// Git SSH URL schema
    /// Repository field must be populated
    Git,

    /// Direct HTTP accessor
    /// Endpoints should be list of URL to directly access those files
    Direct,
}

#[derive(Deserialize)]
pub struct Source {
    /// Type of source or method of access
    pub r#type: SourceType,

    /// Mandatory if Git is selected as the type
    pub repository: Option<String>,

    /// Mandatory if Direct is selected as the type
    pub endpoints: Option<Vec<String>>,
}
