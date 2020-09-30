use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::str::from_utf8;

use serde::{Deserialize, Serialize};

use spdcp::Comment;

use schema::symbol::{
    parse_type_signature, Constant, Define, DocLocation, Documentation, EnumStruct, Enumeration,
    Field, Function, MethodMap, Property, Type, TypeDefinition, TypeSet,
};

mod error;

use error::{AlternatorError, Result};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Deserialize, Serialize, Default)]
pub struct AlternatorStrand {
    pub functions: HashMap<String, Function>,

    pub methodmaps: HashMap<String, MethodMap>,

    pub enumstructs: HashMap<String, EnumStruct>,

    pub constants: HashMap<String, Constant>,

    pub defines: HashMap<String, Define>,

    pub enums: HashMap<String, Enumeration>,

    pub typesets: HashMap<String, TypeSet>,

    pub typedefs: HashMap<String, TypeDefinition>,
}

#[derive(Deserialize)]
pub struct DPStrand {
    pub functions: Vec<Function>,

    pub methodmaps: Vec<MethodMap>,

    pub enumstructs: Vec<EnumStruct>,

    pub constants: Vec<Constant>,

    pub defines: Vec<Define>,

    pub enums: Vec<Enumeration>,

    pub typesets: Vec<TypeSet>,

    pub typedefs: Vec<TypeDefinition>,
}

pub async fn consume<T: Into<Vec<u8>>>(atom: T, content: Vec<u8>) -> Result<AlternatorStrand> {  
    let dp_ptr: *const c_char = unsafe {
        parse(
            CString::new(content.clone())?.as_ptr(),
            CString::new(atom.into())?.as_ptr(),
        )
    };

    let parsed = unsafe {
        CStr::from_ptr(dp_ptr.as_ref().ok_or(AlternatorError::ParseFail)?).to_string_lossy()
    };

    Ok(DPStrand::parse(&parsed, from_utf8(&content)?).await?)
}

impl DPStrand {
    pub async fn parse(parsed: &str, content: &str) -> Result<AlternatorStrand> {
        let mut dp_strand: Self = serde_json::from_str(parsed)?;

        let mut alternator_strand = AlternatorStrand::default();

        for m in &mut dp_strand.methodmaps {
            Self::process_methodmap(m, &content).await;

            alternator_strand
                .methodmaps
                .insert(m.declaration.name.clone(), m.clone());
        }

        for e in &mut dp_strand.enumstructs {
            Self::process_enumstruct(e, &content).await;

            alternator_strand
                .enumstructs
                .insert(e.declaration.name.clone(), e.clone());
        }

        for func in &mut dp_strand.functions {
            Self::process_function(func, &content).await;

            alternator_strand
                .functions
                .insert(func.declaration.name.clone(), func.clone());
        }

        for constant in &mut dp_strand.constants {
            Self::process_constant(constant, &content).await;

            alternator_strand
                .constants
                .insert(constant.declaration.name.clone(), constant.clone());
        }

        for define in &mut dp_strand.defines {
            Self::process_define(define, &content).await;

            alternator_strand
                .defines
                .insert(define.declaration.name.clone(), define.clone());
        }

        for r#enum in &mut dp_strand.enums {
            Self::process_enum(r#enum, &content).await;

            alternator_strand
                .enums
                .insert(r#enum.declaration.name.clone(), r#enum.clone());
        }

        for typeset in &mut dp_strand.typesets {
            Self::process_typeset(typeset, &content).await;

            alternator_strand
                .typesets
                .insert(typeset.declaration.name.clone(), typeset.clone());
        }

        for typedef in &mut dp_strand.typedefs {
            Self::process_typedef(typedef, &content).await;

            alternator_strand
                .typedefs
                .insert(typedef.declaration.name.clone(), typedef.clone());
        }

        Ok(alternator_strand)
    }

    async fn process_methodmap(m: &mut MethodMap, section: &str) {
        Self::process_section(&mut m.declaration.documentation, section).await;

        for method in &mut m.methods {
            Self::process_function(method, section).await;
        }

        for property in &mut m.properties {
            Self::process_property(property, section).await;
        }
    }

    async fn process_enumstruct(e: &mut EnumStruct, section: &str) {
        Self::process_section(&mut e.declaration.documentation, section).await;

        for method in &mut e.methods {
            Self::process_function(method, section).await;
        }

        for field in &mut e.fields {
            Self::process_field(field, section).await;
        }
    }

    async fn process_typeset(t: &mut TypeSet, section: &str) {
        Self::process_section(&mut t.declaration.documentation, section).await;

        for type_t in &mut t.types {
            Self::process_type(type_t, section).await;
        }
    }

    async fn process_enum(e: &mut Enumeration, section: &str) {
        Self::process_section(&mut e.declaration.documentation, section).await;

        for entry in &mut e.entries {
            Self::process_section(&mut entry.declaration.documentation, section).await;
        }
    }

    async fn process_function(f: &mut Function, section: &str) {
        Self::process_section(&mut f.declaration.documentation, section).await;

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
        Self::process_section(&mut p.declaration.documentation, section).await;
    }

    async fn process_field(f: &mut Field, section: &str) {
        Self::process_section(&mut f.declaration.documentation, section).await;
    }

    async fn process_constant(c: &mut Constant, section: &str) {
        Self::process_section(&mut c.declaration.documentation, section).await;
    }

    async fn process_define(d: &mut Define, section: &str) {
        Self::process_section(&mut d.declaration.documentation, section).await;
    }

    async fn process_type(t: &mut Type, section: &str) {
        Self::process_section(&mut t.documentation, section).await;

        t.parsed_signature = Some(parse_type_signature(&t.r#type));
    }

    async fn process_typedef(t: &mut TypeDefinition, section: &str) {
        Self::process_section(&mut t.declaration.documentation, section).await;

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

        let section: String = from_utf8(snippet).unwrap().to_owned();

        println!("{}", section);

        doc.docs = Some(Comment::parse(section));
    }
}
