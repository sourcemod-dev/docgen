use serde::{Deserialize, Serialize};

use crate::symbol::{Declaration, Function};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Type of the field
    pub r#type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumStruct {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Functions within this enum struct
    pub methods: Vec<Function>,

    /// Fields within this enum struct
    pub fields: Vec<Field>,
}
