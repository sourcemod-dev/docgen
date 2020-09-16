use serde::{Deserialize, Serialize};

use crate::symbol::{
    Constant, Define, EnumStruct, Enumeration, Function, MethodMap, TypeDefinition, TypeSet,
};

/// Structural representation of an include's file contents
#[derive(Debug, Serialize, Deserialize)]
pub struct IncludeFile {
    pub functions: Vec<Function>,
    pub methodmaps: Vec<MethodMap>,
    pub enumstructs: Vec<EnumStruct>,
    pub constants: Vec<Constant>,
    pub defines: Vec<Define>,
    pub enums: Vec<Enumeration>,
    pub typesets: Vec<TypeSet>,
    pub typedefs: Vec<TypeDefinition>,
}
