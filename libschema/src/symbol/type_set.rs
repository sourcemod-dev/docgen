use std::ops::ShlAssign;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::symbol::{Declaration, Documentation, TypeSignature};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Type {
    #[serde(flatten)]
    pub documentation: Documentation,

    /// Signature of the function
    pub r#type: String,

    /// Parsed function signature
    pub parsed_signature: Option<TypeSignature>,
}

impl ShlAssign for Type {
    fn shl_assign(&mut self, rhs: Self) {
        self.documentation <<= rhs.documentation;
        self.r#type = rhs.r#type;
        self.parsed_signature = rhs.parsed_signature;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeSet {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Type signatures
    pub types: HashMap<String, Type>,
}

impl ShlAssign for TypeSet {
    fn shl_assign(&mut self, rhs: Self) {
        self.declaration <<= rhs.declaration;
        self.types = rhs.types;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DPTypeSet {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Type signatures
    pub types: Vec<Type>,
}
