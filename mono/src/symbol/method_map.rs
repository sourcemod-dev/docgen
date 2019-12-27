use serde::{Serialize, Deserialize};

use crate::symbol::{Declaration, Function, Property};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodMap {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Functions within this methodmap
    pub methods: Vec<Function>,

    /// Properties within this methodmap
    pub properties: Vec<Property>,
}
