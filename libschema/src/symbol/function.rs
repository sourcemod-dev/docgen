use std::ops::ShlAssign;

use serde::{Deserialize, Serialize};

use crate::symbol::{Argument, Declaration};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    #[serde(flatten)]
    pub declaration: Declaration,

    /// Ex kinds: (forward, stock, etc)
    pub kind: String,

    /// Return type of the function
    pub return_type: String,

    /// Arguments of the function
    pub arguments: Vec<Argument>,
}

impl ShlAssign for Function {
    fn shl_assign(&mut self, rhs: Self) {
        self.declaration <<= rhs.declaration;
        self.kind = rhs.kind;
        self.return_type = rhs.return_type;
        self.arguments = rhs.arguments;
    }
}
