use serde::{Serialize, Deserialize};

use crate::symbol::{Declaration, Function, Property};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodMap {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Parent inheritance if any
    pub parent: Option<String>,

    /// Functions within this methodmap
    pub methods: Vec<Function>,

    /// Properties within this methodmap
    pub properties: Vec<Property>,
}
