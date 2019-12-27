use serde::{Serialize, Deserialize};

use crate::symbol::Declaration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDefinition {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Function definition
    pub r#type: String,
}
