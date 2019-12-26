use serde::{Serialize, Deserialize};

use crate::symbol::{Declaration, Function, Property};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodMap {
    #[serde(flatten)]
    pub declaration: Declaration,

    pub methods: Vec<Function>,
    pub properties: Vec<Property>,
}
