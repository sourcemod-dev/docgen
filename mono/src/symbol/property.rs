use serde::{Deserialize, Serialize};

use crate::symbol::Declaration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Type of the property
    pub r#type: String,

    /// Whether getter exists
    pub getter: bool,

    /// Whether setter exists
    pub setter: bool,
}
