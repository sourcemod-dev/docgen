use serde::{Serialize, Deserialize};

use crate::symbol::{Declaration, Argument};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
