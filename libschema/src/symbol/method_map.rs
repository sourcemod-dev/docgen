use std::collections::HashMap;
use std::ops::ShlAssign;

use serde::{Deserialize, Serialize};

use crate::metadata::Metadata;
use crate::symbol::{Declaration, Function, Metable, Property};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodMap {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Parent inheritance if any
    pub parent: Option<String>,

    /// Functions within this methodmap
    pub methods: HashMap<String, Function>,

    /// Properties within this methodmap
    pub properties: HashMap<String, Property>,
}

impl Metable for MethodMap {
    fn metadata(&mut self) -> &mut Option<Metadata> {
        &mut self.declaration.documentation.metadata
    }
}

impl ShlAssign for MethodMap {
    fn shl_assign(&mut self, rhs: Self) {
        self.declaration <<= rhs.declaration;
        self.parent = rhs.parent;
        self.methods = rhs.methods;
        self.properties = rhs.properties;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DPMethodMap {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Parent inheritance if any
    pub parent: Option<String>,

    /// Functions within this methodmap
    pub methods: Vec<Function>,

    /// Properties within this methodmap
    pub properties: Vec<Property>,
}
