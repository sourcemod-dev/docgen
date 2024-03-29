use std::ops::ShlAssign;

use serde::{Deserialize, Serialize};

use crate::metadata::Metadata;
use crate::symbol::{Declaration, Metable};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl Metable for Property {
    fn metadata(&mut self) -> &mut Option<Metadata> {
        &mut self.declaration.documentation.metadata
    }
}

impl ShlAssign for Property {
    fn shl_assign(&mut self, rhs: Self) {
        self.declaration <<= rhs.declaration;
        self.r#type = rhs.r#type;
        self.getter = rhs.getter;
        self.setter = rhs.setter;
    }
}
