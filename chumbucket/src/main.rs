#![feature(trait_alias)]

use anyhow::Result;
use clap::{crate_authors, crate_description, crate_version, App, Arg};

use schema::{
    bundle::Bundle,
    manifest::{Manifest, SourceType},
};
use walker::Walker;

mod accessors;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("Chum Bucket")
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("include")
                .about("Target file is an include file")
                .short('i'),
        )
        .arg(
            Arg::with_name("rebuild-history")
                .about("Rebuild versioning history from the start. Will not read existing bundle for versioning.")
                .long("rebuild-history")
                .required(false),
        )
        .arg(
            Arg::with_name("bundle")
                .about("Existing bundle to continue from")
                .short('b')
                .takes_value(true)
                .required(false)
        )
        .arg(
            // By default, it will output to a path relative to the chumbucket
            Arg::with_name("output")
                .about("Location to output bundle to")
                .short('o')
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("file")
                .about("Input file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let fs_content = std::fs::read(matches.value_of("file").unwrap())?;

    let fs_out = matches.value_of("output").unwrap();

    let input = std::str::from_utf8(&fs_content)?;

    // Supercede and process singular include only
    if matches.is_present("include") {
        let res = alternator::consume("chumbucket", input).await?;

        write_to_disk(fs_out, res)?;

        return Ok(());
    }

    let manifest: Manifest = toml::from_slice(&fs_content)?;
    let mut bundle: Option<Bundle> = None;

    if matches.is_present("bundle") {
        let bundle_str = std::fs::read(matches.value_of("bundle").unwrap())?;

        bundle = Some(serde_json::from_slice(&bundle_str)?);
    }

    match manifest.source.r#type {
        // SourceType::Git => {

        // },
        _ => {
            let repo = manifest.source.repository.clone().unwrap();
            let patterns = manifest.source.patterns.clone().unwrap();

            let mut walker = Walker::from_remote(&repo, &manifest.meta.name, patterns)?;

            let git = accessors::Git::from_walker(&manifest, None, &mut walker)?;
        }
    };

    Ok(())
}

fn write_to_disk<T>(loc: &str, t: T) -> Result<()>
where
    T: serde::Serialize,
{
    let s = serde_json::to_string(&t)?;

    std::fs::write(loc, &s)?;

    Ok(())
}
