use std::path::PathBuf;

use anyhow::{anyhow, Result};

use clap::ArgMatches;

use schema::{
    bundle::Bundle,
    index::{Index, IndexMap},
};

use crate::utils::write_to_disk;

pub async fn index_command(matches: &ArgMatches) -> Result<()> {
    let dir = std::fs::read_dir(matches.value_of("directory").unwrap())?;

    let fs_out = matches.value_of("output").unwrap();

    let mut index_map = IndexMap::new();

    if matches.is_present("index") {
        let index_io = std::fs::read(matches.value_of("index").unwrap())?;

        let parsed_indices: IndexMap = serde_json::from_slice(&index_io)?;

        index_map = parsed_indices;
    }

    let entries: Vec<PathBuf> = dir
        .filter(|i| i.is_ok())
        .map(|i| i.unwrap())
        .filter(|i| i.path().extension().is_some())
        .filter(|i| i.path().extension().unwrap() == "bundle")
        .map(|i| i.path())
        .collect::<Vec<_>>();

    let mut diffs = 0u64;

    for entry in &entries {
        let file_stem = entry
            .file_stem()
            .ok_or(anyhow!("Missing file stem"))?
            .to_string_lossy()
            .to_string();

        let content = std::fs::read(entry)?;

        let bundle: Bundle = serde_json::from_slice(&content)?;

        match index_map.get(&bundle.meta.name) {
            Some(v) => {
                if v.meta != bundle.meta || v.source != bundle.source {
                    diffs += 1;

                    index_map.insert(
                        bundle.meta.name.clone(),
                        Index {
                            meta: bundle.meta,
                            source: bundle.source,
                            file_stem,
                        },
                    );
                }
            }
            None => {
                diffs += 1;

                index_map.insert(
                    bundle.meta.name.clone(),
                    Index {
                        meta: bundle.meta,
                        source: bundle.source,
                        file_stem,
                    },
                );
            }
        }
    }

    if diffs > 0 {
        write_to_disk(fs_out, index_map)?;
    }

    Ok(())
}
