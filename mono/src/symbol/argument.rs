use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Argument {
    pub r#type: String,
    pub name: String,
    pub decl: String,
}
