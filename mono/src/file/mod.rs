use serde::{Serialize, Deserialize};

use crate::symbol::{
    Function,
    MethodMap,
    Constant,
    Enumeration,
    TypeSet,
    TypeDefinition,
};

/// Structural representation of an include's file contents
#[derive(Debug, Serialize, Deserialize)]
pub struct IncludeFile {
    pub functions: Vec<Function>,
    pub methodmaps: Vec<MethodMap>,
    pub constants: Vec<Constant>,
    pub enums: Vec<Enumeration>,
    pub typesets: Vec<TypeSet>,
    pub typedefs: Vec<TypeDefinition>,
}
