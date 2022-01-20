use std::ops::ShlAssign;
use std::collections::HashMap;

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

impl ShlAssign for Field {
    fn shl_assign(&mut self, rhs: Self) {
        self.declaration <<= rhs.declaration;
        self.r#type = rhs.r#type;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumStruct {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Functions within this enum struct
    pub methods: HashMap<String, Function>,

    /// Fields within this enum struct
    pub fields: HashMap<String, Field>,
}

impl ShlAssign for EnumStruct {
    fn shl_assign(&mut self, rhs: Self) {
        self.declaration <<= rhs.declaration;
        self.methods = rhs.methods;
        self.fields = rhs.fields;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DPEnumStruct {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Functions within this enum struct
    pub methods: Vec<Function>,

    /// Fields within this enum struct
    pub fields: Vec<Field>,
}
