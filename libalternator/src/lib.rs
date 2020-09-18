use std::ffi::{CStr, CString};

use serde::Deserialize;

use schema::symbol::{
    Function, MethodMap, Enumeration, Constant, Define, Enumeration, TypeSet, TypeDefinition,
};

mod error;

use error::Result;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Deserialize)]
pub struct AlternatorStrand {
    pub functions: Vec<Function>,

    pub methodmaps: Vec<MethodMap>,

    pub enumstructs: Vec<EnumStruct>,

    pub constants: Vec<Constant>,

    pub defines: Vec<Define>,

    pub enums: Vec<Enumeration>,

    pub typesets: Vec<TypeSet>,

    pub typedefs: Vec<TypeDefinition>,
}

pub async fn parse<T: Into<Vec<u8>>>(atom: T, content: T) {
    
}
