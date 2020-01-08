use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, File};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{from_str, to_writer};

use reqwest::get;

use crate::errors::Result;
use crate::parser::parse_documentation;

use mono::file::IncludeFile;
use mono::manifest::IncludeManifest;

pub async fn process_manifest(file: PathBuf, output: Option<PathBuf>, base_url: String) {
    let content = read_to_string(&file).expect("Unable to read from file");

    let include_pairs: HashMap<String, String> =
        from_str(&content).expect("Unable to parse manifest file");

    let mut output_path = PathBuf::new();

    let stem: String = file.file_stem().unwrap().to_string_lossy().to_string();

    if let Some(output_some) = &output {
        output_path.push(output_some);
    } else {
        output_path.push(file.file_stem().unwrap().to_os_string());
    }

    create_dir_all(&output_path).expect("Unable to create output directory");

    let mut manifest_include_pairs: HashMap<String, String> =
        HashMap::with_capacity(include_pairs.keys().len());

    for (k, v) in include_pairs {
        let result = process_entry(k, v).await;

        match result {
            Ok((k, v)) => {
                let mut local_path = output_path.clone();
                local_path.push(format!("{}.gid", &k));

                let file = match File::create(&local_path) {
                    Ok(f) => f,
                    Err(e) => {
                        println!("Failed to create file for {}: {}", &k, e);
                        continue;
                    }
                };

                match to_writer(file, &v) {
                    Ok(_) => {
                        manifest_include_pairs
                            .insert(k.clone(), format!("{}/{}/{}.gid", base_url, stem, &k));
                    }
                    Err(e) => {
                        println!("Failed to write to file for {}: {}", &k, e);
                    }
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    let mut manifest_path = output_path.clone();
    manifest_path.push("include.manifest");

    let manifest_file = File::create(&manifest_path).expect("Unable to create manifest file");

    to_writer(
        manifest_file,
        &IncludeManifest {
            includes: manifest_include_pairs,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time broke")
                .as_secs(),
        },
    )
    .expect("Unable to write manifest file");
}

async fn process_entry(key: String, endpoint: String) -> Result<(String, IncludeFile)> {
    let resp: String = get(&endpoint)?.text()?;

    let include = match parse_documentation(&resp).await {
        Ok(v) => v,
        Err(v) => {
            return Err(v);
        }
    };

    Ok((key, include))
}
