use serde::{Serialize, Deserialize};

use crate::symbol::{Documentation, Declaration};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Type {
    pub r#type: String,

    #[serde(flatten)]
    pub documentation: Documentation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeSet {
    #[serde(flatten)]
    pub declaration: Declaration,

    pub types: Vec<Type>,
}
