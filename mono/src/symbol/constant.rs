use serde::{Serialize, Deserialize};

use crate::symbol::Declaration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Constant {
    #[serde(flatten)]
    pub declaration: Declaration,
}
