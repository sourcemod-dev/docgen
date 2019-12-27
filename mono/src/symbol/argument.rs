use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Argument {
    /// Type of the argument
    pub r#type: String,

    // Name of the argument
    pub name: String,

    /// Raw declaration of the argument
    pub decl: String,
    
    /// Default value if any
    pub default: Option<String>,
}
