use std::collections::HashMap;
use std::ops::ShlAssign;

use serde::{Deserialize, Serialize};

use crate::metadata::Metadata;
use crate::symbol::{Declaration, Documentation, Metable, TypeSignature};

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

impl Metable for Type {
    fn metadata(&mut self) -> &mut Option<Metadata> {
        &mut self.documentation.metadata
    }
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

impl Metable for TypeSet {
    fn metadata(&mut self) -> &mut Option<Metadata> {
        &mut self.declaration.documentation.metadata
    }
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
