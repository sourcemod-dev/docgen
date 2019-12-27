use serde::{Serialize, Deserialize};

use crate::symbol::{Documentation, Declaration};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Type {
    /// Signature of the function
    pub r#type: String,

    #[serde(flatten)]
    pub documentation: Documentation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeSet {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Type signatures
    pub types: Vec<Type>,
}
