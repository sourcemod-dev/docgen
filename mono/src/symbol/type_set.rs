use serde::{Serialize, Deserialize};

use crate::symbol::{Documentation, Declaration, TypeSignature};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Type {
    #[serde(flatten)]
    pub documentation: Documentation,

    /// Signature of the function
    pub r#type: String,

    /// Parsed function signature
    pub parsed_signature: Option<TypeSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeSet {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Type signatures
    pub types: Vec<Type>,
}
