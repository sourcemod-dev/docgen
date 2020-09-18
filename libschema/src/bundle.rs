use serde::{Deserialize, Serialize};

use super::meta::Meta;

use crate::symbol::{
    Constant, Define, EnumStruct, Enumeration, Function, MethodMap, TypeDefinition, TypeSet,
};

#[derive(Deserialize, Serialize)]
pub struct Bundle {
    /// Meta descriptor of bundle content
    pub meta: Meta,

    pub strands: Vec<Strand>,

    /// Current version this bundle was parsed from
    /// Chum bucket will continue from this commit
    pub version: Versioning,
}

#[derive(Deserialize, Serialize)]
pub struct Strand {
    pub functions: Vec<Fiber<Function>>,

    pub methodmaps: Vec<Fiber<MethodMap>>,

    pub enumstructs: Vec<Fiber<EnumStruct>>,

    pub constants: Vec<Fiber<Constant>>,

    pub defines: Vec<Fiber<Define>>,

    pub enums: Vec<Fiber<Enumeration>>,

    pub typesets: Vec<Fiber<TypeSet>>,

    pub typedefs: Vec<Fiber<TypeDefinition>>,
}

#[derive(Deserialize, Serialize)]
pub struct Fiber<T> {
    pub symbol: T,

    /// SVN version this symbol was introduced
    pub created: Option<Versioning>,

    /// SVN version this symbol was last modified
    pub last_updated: Option<Versioning>,
}

#[derive(Deserialize, Serialize)]
pub struct Versioning {
    pub hash: String,

    pub time: u64,
}
