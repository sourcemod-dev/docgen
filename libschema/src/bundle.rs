use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::manifest::Source;
use super::meta::Meta;
use super::metadata::Versioning;

use crate::symbol::{
    Constant, Define, EnumStruct, Enumeration, Function, MethodMap, TypeDefinition, TypeSet,
};

#[derive(Deserialize, Serialize)]
pub struct Bundle {
    /// Meta descriptor of bundle content
    pub meta: Meta,

    /// Manifest source
    pub source: Source,

    /// Strand or each individual include file
    /// With optional addon metadata for versioninig
    pub strands: HashMap<String, Strand>,

    /// Current version this bundle was last parsed from
    /// Chum bucket will continue from this commit
    pub version: Option<Versioning>,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct Strand {
    pub functions: HashMap<String, Function>,

    pub methodmaps: HashMap<String, MethodMap>,

    pub enumstructs: HashMap<String, EnumStruct>,

    pub constants: HashMap<String, Constant>,

    pub defines: HashMap<String, Define>,

    pub enums: HashMap<String, Enumeration>,

    pub typesets: HashMap<String, TypeSet>,

    pub typedefs: HashMap<String, TypeDefinition>,
}
