use std::ops::ShlAssign;

use serde::{Deserialize, Serialize};

use crate::symbol::Declaration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Constant {
    #[serde(flatten)]
    pub declaration: Declaration,
}

impl ShlAssign for Constant {
    fn shl_assign(&mut self, rhs: Self) {
        self.declaration <<= rhs.declaration;
    }
}
