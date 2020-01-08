use std::ffi::{CStr, CString};

use serde_json::from_str;

use mono::file::IncludeFile;
use mono::symbol::{
    parse_type_signature, Constant, DocLocation, Documentation, EnumStruct, Enumeration, Field,
    Function, MethodMap, Property, Type, TypeDefinition, TypeSet,
};

use spdcp::Comment;

use crate::errors::{Error, Result};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub async fn parse_documentation<D: Into<String>>(data: D) -> Result<IncludeFile> {
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

    for m in &mut include_file.methodmaps {
        process_methodmap(m, &raw_str).await;
    }

    for e in &mut include_file.enumstructs {
        process_enumstruct(e, &raw_str).await;
    }

    for func in &mut include_file.functions {
        process_function(func, &raw_str).await;
    }

    for constant in &mut include_file.constants {
        process_constant(constant, &raw_str).await;
    }

    for r#enum in &mut include_file.enums {
        process_enum(r#enum, &raw_str).await;
    }

    for typeset in &mut include_file.typesets {
        process_typeset(typeset, &raw_str).await;
    }

    for typedef in &mut include_file.typedefs {
        process_typedef(typedef, &raw_str).await;
    }

    Ok(include_file)
}

async fn process_methodmap(m: &mut MethodMap, section: &str) {
    process_section(&mut m.declaration.documentation, section).await;

    for method in &mut m.methods {
        process_function(method, section).await;
    }

    for property in &mut m.properties {
        process_property(property, section).await;
    }
}

async fn process_enumstruct(e: &mut EnumStruct, section: &str) {
    process_section(&mut e.declaration.documentation, section).await;

    for method in &mut e.methods {
        process_function(method, section).await;
    }

    for field in &mut e.fields {
        process_field(field, section).await;
    }
}

async fn process_typeset(t: &mut TypeSet, section: &str) {
    process_section(&mut t.declaration.documentation, section).await;

    for type_t in &mut t.types {
        process_type(type_t, section).await;
    }
}

async fn process_enum(e: &mut Enumeration, section: &str) {
    process_section(&mut e.declaration.documentation, section).await;

    for entry in &mut e.entries {
        process_section(&mut entry.declaration.documentation, section).await;
    }
}

async fn process_function(f: &mut Function, section: &str) {
    process_section(&mut f.declaration.documentation, section).await;

    // For array types, the array couples the type in the
    // `type` prop, but in practice, it should couple name
    // Instead of parsing type and extracting any dimension out of type
    // we extract it directly from decl which already is correct
    // for arg in &mut f.arguments {
    //     if arg.decl.contains("...") {
    //         arg.r#type = arg.decl.replace("...", "");
    //         arg.name = "...".to_string();
    //     } else {
    //         let split = arg.decl.split(" ").collect::<Vec<_>>();

    //         if split.len() == 2 {
    //             arg.r#type = split[0].to_string();
    //             arg.name = split[1].to_string();
    //         } else if split.len() > 2 {
    //             arg.r#type = split[0..2].join(" ");
    //             arg.name = split[2].to_string();
    //         }
    //     }
    // }
}

async fn process_property(p: &mut Property, section: &str) {
    process_section(&mut p.declaration.documentation, section).await;
}

async fn process_field(f: &mut Field, section: &str) {
    process_section(&mut f.declaration.documentation, section).await;
}

async fn process_constant(c: &mut Constant, section: &str) {
    process_section(&mut c.declaration.documentation, section).await;
}

async fn process_type(t: &mut Type, section: &str) {
    process_section(&mut t.documentation, section).await;

    t.parsed_signature = Some(parse_type_signature(&t.r#type));
}

async fn process_typedef(t: &mut TypeDefinition, section: &str) {
    process_section(&mut t.declaration.documentation, section).await;

    t.parsed_signature = Some(parse_type_signature(&t.r#type));
}

async fn process_section(doc: &mut Documentation, section: &str) {
    if doc.docs != None {
        return;
    }

    if doc.doc_start == DocLocation::from(0) || doc.doc_end == DocLocation::from(0) {
        return;
    }

    let bytes = section.as_bytes();

    let start: usize = doc.doc_start.into();
    let end: usize = doc.doc_end.into();

    let snippet = &bytes[start..end];

    let section: String = std::str::from_utf8(snippet).unwrap().to_owned();

    doc.docs = Some(Comment::parse(section));
}
