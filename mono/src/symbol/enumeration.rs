use serde::{Serialize, Deserialize};

use crate::symbol::Declaration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enumeration {
    #[serde(flatten)]
	pub declaration: Declaration,

	pub entries: Vec<Declaration>,
}
