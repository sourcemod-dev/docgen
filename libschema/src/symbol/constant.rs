use std::ops::ShlAssign;

use serde::{Deserialize, Serialize};

use crate::metadata::Metadata;
use crate::symbol::{Declaration, Metable};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Constant {
    #[serde(flatten)]
    pub declaration: Declaration,
}

impl Metable for Constant {
    fn metadata(&mut self) -> &mut Option<Metadata> {
        &mut self.declaration.documentation.metadata
    }
}

impl ShlAssign for Constant {
    fn shl_assign(&mut self, rhs: Self) {
        self.declaration <<= rhs.declaration;
    }
}
