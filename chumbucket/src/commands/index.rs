use crate::utils::write_to_disk;
use anyhow::Result;
use clap::ArgMatches;
use schema::{
    bundle::Bundle,
    index::{Index, Indices},
};

pub async fn index_command(matches: &ArgMatches) -> Result<()> {
    let dir = std::fs::read_dir(matches.value_of("directory").unwrap())?;

    let fs_out = matches.value_of("output").unwrap();

    let entries = dir
        .filter(|i| i.is_ok())
        .map(|i| i.unwrap())
        .filter(|i| i.path().extension().is_some())
        .filter(|i| i.path().extension().unwrap() == "bundle")
        .map(|i| i.path())
        .collect::<Vec<_>>();

    let mut indices = Indices::new();

    for entry in &entries {
        let content = std::fs::read(entry)?;

        let bundle: Bundle = serde_json::from_slice(&content)?;

        indices.push(Index {
            meta: bundle.meta,
            source: bundle.source,
        });
    }

    write_to_disk(fs_out, indices)?;

    Ok(())
}
