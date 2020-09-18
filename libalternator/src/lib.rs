use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use serde::Deserialize;

use schema::bundle::Strand;
use schema::symbol::{
    Constant, Define, EnumStruct, Enumeration, Function, MethodMap, TypeDefinition, TypeSet,
};

mod error;

use error::{AlternatorError, Result};

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

pub async fn consume<T: Into<Vec<u8>>>(atom: T, content: T) -> Result<Strand> {
    let dp_ptr: *const c_char = unsafe {
        parse(
            CString::new(content)?.as_ptr(),
            CString::new(atom)?.as_ptr(),
        )
    };

    let parsed = unsafe {
        CStr::from_ptr(dp_ptr.as_ref().ok_or(AlternatorError::ParseFail)?).to_string_lossy()
    };

    let mut alternator_strand: AlternatorStrand = serde_json::from_str(&parsed)?;

    unimplemented!()
}
