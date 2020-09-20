use serde::{Deserialize, Serialize};

use super::meta::Meta;

use crate::symbol::{
    Constant, Define, EnumStruct, Enumeration, Function, MethodMap, TypeDefinition, TypeSet,
};

#[derive(Deserialize, Serialize)]
pub struct Bundle {
    /// Meta descriptor of bundle content
    pub meta: Meta,

    /// Strand or each individual include file
    /// With optional addon metadata for versioninig
    pub strands: Vec<Strand>,

    /// Current version this bundle was last parsed from
    /// Chum bucket will continue from this commit
    pub version: Versioning,
}

#[derive(Deserialize, Serialize)]
pub struct Strand {
    pub functions: Fibers<Function>,

    pub methodmaps: Fibers<MethodMap>,

    pub enumstructs: Fibers<EnumStruct>,

    pub constants: Fibers<Constant>,

    pub defines: Fibers<Define>,

    pub enums: Fibers<Enumeration>,

    pub typesets: Fibers<TypeSet>,

    pub typedefs: Fibers<TypeDefinition>,
}

pub type Fibers<T> = Vec<Fiber<T>>;

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

    /// Walker commit depth
    pub sequence: usize,

    pub time: u64,
}
