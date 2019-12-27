use std::ffi::{CString, CStr};

use serde_json::from_str;

use mono::file::IncludeFile;
use mono::symbol::{
    DocLocation,
    Documentation,
};

use spdcp::Comment;

use futures::future::join_all;

use crate::errors::{Error, Result};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub async fn parse_documentation<D: Into<String>>(
    data: D,
) -> Result<IncludeFile> {
    let raw_str = data.into();

    let content = CString::new(raw_str.clone())?;

    // We are required to pass an unique atom id for completion sake
    let unique_path = CString::new("random_path")?;

    let parsed_ptr = unsafe { parse(content.as_ptr(), unique_path.as_ptr()) };

    let parsed = unsafe {
        match parsed_ptr.as_ref() {
            Some(d) => CStr::from_ptr(d).to_string_lossy().into_owned(),
            None => {
                println!("Failed to parse initial documentation");

                return Err(Error::ParseFailed);
            }
        }
    };

    let mut include_file: IncludeFile = match from_str(&parsed) {
        Ok(v) => v,
        Err(v) => return Err(Error::SchemaMismatch(v)),
    };

    let mut promises = Vec::new();

    for m in &mut include_file.methodmaps {
        promises.push(process_section(&mut m.declaration.documentation, &raw_str));

        for func in &mut m.methods {
            promises.push(process_section(&mut func.declaration.documentation, &raw_str));
        }

        for prop in &mut m.properties {
            promises.push(process_section(&mut prop.declaration.documentation, &raw_str));
        }
    }

    for func in &mut include_file.functions {
        promises.push(process_section(&mut func.declaration.documentation, &raw_str));
    }

    for constant in &mut include_file.constants {
        promises.push(process_section(&mut constant.declaration.documentation, &raw_str));
    }

    for enums in &mut include_file.enums {
        promises.push(process_section(&mut enums.declaration.documentation, &raw_str));

        for entry in &mut enums.entries {
            promises.push(process_section(&mut entry.documentation, &raw_str));
        }
    }

    for typeset in &mut include_file.typesets {
        promises.push(process_section(&mut typeset.declaration.documentation, &raw_str));

        for type_e in &mut typeset.types {
            promises.push(process_section(&mut type_e.documentation, &raw_str));
        }
    }

    for typedef in &mut include_file.typedefs {
        promises.push(process_section(&mut typedef.declaration.documentation, &raw_str));
    }

    {
        join_all(promises).await;
    }

    Ok(include_file)
}

async fn process_section<S>(doc: &mut Documentation, section: S)
where
    S: Into<String>,
{
    if doc.docs != None {
        return
    }

    if doc.doc_start == DocLocation::from(0) || doc.doc_end == DocLocation::from(0) {
        return
    }

    let byte_vec: Vec<u8> = section.into().into_bytes();

    let start: usize = doc.doc_start.into();
    let end: usize = doc.doc_end.into();

    let snippet = &byte_vec[start .. end];

    let section: String = std::str::from_utf8(snippet).unwrap().to_owned();

    doc.docs = Some(Comment::parse(section));
}
