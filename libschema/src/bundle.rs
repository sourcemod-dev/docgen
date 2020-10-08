use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::meta::Meta;

use crate::symbol::{
    Constant, Define, EnumStruct, Enumeration, Function, MethodMap, TypeDefinition, TypeSet,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct Bundle {
    /// Meta descriptor of bundle content
    pub meta: Meta,

    /// Strand or each individual include file
    /// With optional addon metadata for versioninig
    pub strands: HashMap<String, Strand>,

    /// Current version this bundle was last parsed from
    /// Chum bucket will continue from this commit
    pub version: Option<Versioning>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
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

pub type Fibers<T> = HashMap<String, Fiber<T>>;

#[derive(Deserialize, Serialize, Debug)]
pub struct Fiber<T> {
    pub symbol: T,

    /// SVN version this symbol was introduced
    pub created: Option<Versioning>,

    /// SVN version this symbol was last modified
    pub last_updated: Option<Versioning>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Versioning {
    pub hash: String,

    /// Rev-list count
    /// Mainly used for core where product.version will be within spec paths
    pub count: u64,

    pub time: i64,
}
