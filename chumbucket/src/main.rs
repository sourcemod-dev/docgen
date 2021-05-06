#![feature(trait_alias)]

use anyhow::Result;

use clap::{crate_authors, crate_description, crate_version, App, Arg};

mod accessors;
mod commands;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("Chum Bucket")
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .subcommand(
            App::new("build-index")
                .about("Generate an index file from a directory of bundles")
                .arg(
                    Arg::new("output")
                        .about("Location to output index to")
                        .short('o')
                        .takes_value(true)
                        .required(true)
                )
                .arg(
                    Arg::new("index")
                        .about("Existing index file for partial check")
                        .short('i')
                        .takes_value(true)
                        .required(false)
                )
                .arg(
                    Arg::new("directory")
                        .about("Bundle directory")
                        .takes_value(true)
                        .required(true)
                )
        )
        .subcommand(
            App::new("generate")
                .about("Generate bundle or alternator strand from manifest/include respectively")
                .arg(
                    Arg::new("include")
                        .about("Target file is an include file")
                        .short('i'),
                )
                .arg(
                    Arg::new("rebuild-history")
                        .about("Rebuild versioning history from the start. Will not read existing bundle for versioning.")
                        .long("rebuild-history")
                        .required(false),
                )
                .arg(
                    Arg::new("bundle")
                        .about("Existing bundle to continue from")
                        .short('b')
                        .takes_value(true)
                        .required(false)
                )
                .arg(
                    // By default, it will output to a path relative to the chumbucket
                    Arg::new("output")
                        .about("Location to output bundle to")
                        .short('o')
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::new("file")
                        .about("Input file")
                        .takes_value(true)
                        .required(true),
                )
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("generate") {
        commands::generate_command(matches).await?
    }

    if let Some(matches) = matches.subcommand_matches("build-index") {
        commands::index_command(matches).await?
    }

    Ok(())
}
