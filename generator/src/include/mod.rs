use std::fs::{
    File,
    read_to_string,
};
use std::path::PathBuf;

use serde_json::to_writer;

use crate::parser::parse_documentation;

pub async fn process_include(file: PathBuf, output: Option<PathBuf>) {
    let content = read_to_string(&file).expect("Unable to read from file");

    let include = match parse_documentation(content).await {
        Ok(v) => v,
        Err(v) => {
            println!("Unable to generate documentation for include file: {}", v);

            return;
        },
    };

    if output.is_some() {
        let out_pathbuf = output.unwrap();

        let out_file = File::create(&out_pathbuf).expect("Unable to create output file");

        to_writer(out_file, &include).expect("Unable to write to output file");

        return;
    }

    let mut out_file_path = file.file_stem().unwrap().to_os_string();
    out_file_path.push(".gid");

    let out_file = File::create(&out_file_path).expect("Unable to create output file");

    to_writer(out_file, &include).expect("Unable to write to output file");
}
