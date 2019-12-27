use spdcp::Comment;

use mono::symbol::{Documentation, DocLocation};
use mono::file::IncludeFile;

use serde_json::{from_str, to_writer};

use structopt::StructOpt;

use std::path::PathBuf;
use std::fs::{read_to_string, File};
use std::ffi::{CStr, CString};

use futures::future::join_all;
use futures::executor::block_on;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    /// Output path of generated include documentation.
    pub output: PathBuf,

    #[structopt(name = "FILE", parse(from_os_str))]
    /// Input path of an include (.inc) file to parse.
    pub input: PathBuf,
}

fn main() {
    let cli = Cli::from_args();

    process_file(cli.input, cli.output);
}

fn process_file(path: PathBuf, output: PathBuf) {
    let o_content = match read_to_string(path) {
        Ok(d) => d,
        Err(_) => {
            return;
        },
    };

    let content = CString::new(o_content.clone()).expect("Uh oh");

    let unique_path = CString::new(o_content.len().to_string()).expect("Oh noes");

    let parsed_ptr = unsafe { parse(content.as_ptr(), unique_path.as_ptr()) };

    let parsed = unsafe {
        match parsed_ptr.as_ref() {
            Some(d) => CStr::from_ptr(d).to_string_lossy().into_owned(),
            None => {
                println!("Failed to parse initial documentation");

                return;
            }
        }
    };

    let mut include_file: IncludeFile = from_str(&parsed).expect("Darn");

    let mut promises = Vec::new();

    for m in &mut include_file.methodmaps {
        promises.push(process_section(&mut m.declaration.documentation, &o_content));

        for func in &mut m.methods {
            promises.push(process_section(&mut func.declaration.documentation, &o_content));
        }

        for prop in &mut m.properties {
            promises.push(process_section(&mut prop.declaration.documentation, &o_content));
        }
    }

    for func in &mut include_file.functions {
        promises.push(process_section(&mut func.declaration.documentation, &o_content));
    }

    for constant in &mut include_file.constants {
        promises.push(process_section(&mut constant.declaration.documentation, &o_content));
    }

    for enums in &mut include_file.enums {
        promises.push(process_section(&mut enums.declaration.documentation, &o_content));

        for entry in &mut enums.entries {
            promises.push(process_section(&mut entry.documentation, &o_content));
        }
    }

    for typeset in &mut include_file.typesets {
        promises.push(process_section(&mut typeset.declaration.documentation, &o_content));

        for type_e in &mut typeset.types {
            promises.push(process_section(&mut type_e.documentation, &o_content));
        }
    }

    for typedef in &mut include_file.typedefs {
        promises.push(process_section(&mut typedef.declaration.documentation, &o_content));
    }

    {
        block_on(join_all(promises));
    }

    let out = File::create(output).expect("Unable to create out file");

    to_writer(out, &include_file).expect("Unable to write to out file");
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
