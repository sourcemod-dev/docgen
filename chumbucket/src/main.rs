#![feature(trait_alias)]

use anyhow::Result;
use clap::{crate_authors, crate_description, crate_version, App, Arg};
use schema::{
    bundle::{Bundle, Fiber, Strand},
    manifest::{Manifest, SourceType},
};
use std::collections::HashMap;
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

    // Supercede and process singular include only
    if matches.is_present("include") {
        let res = alternator::consume("chumbucket", fs_content).await?;

        write_to_disk(fs_out, res)?;

        return Ok(());
    }

    let manifest: Manifest = toml::from_slice(&fs_content)?;
    let mut bundle: Option<Bundle> = None;
    let mut from_time: Option<i64> = None;

    if matches.is_present("bundle") {
        let bundle_str = std::fs::read(matches.value_of("bundle").unwrap())?;

        let parsed_bundle: Bundle = serde_json::from_slice(&bundle_str)?;

        if let Some(version) = &parsed_bundle.version {
            from_time = Some(version.time);
        }

        bundle = Some(parsed_bundle);
    }

    match manifest.source.r#type {
        SourceType::Git => {
            let mut walker = Walker::from_remote(
                &manifest.source.repository.clone().unwrap(),
                &manifest.meta.name,
                manifest.source.patterns.clone().unwrap(),
            )?;

            let git = accessors::Git::from_walker(from_time, &mut walker)?;

            let it_ret = iterate_chronicles(git, manifest, bundle).await?;

            // If there are differences, write to file
            // JSON object keys are not guaranteed to be in ordered each time
            // This check exists to avoid changing of E-tag on CDN proxy
            if it_ret.1 > 0 {
                write_to_disk(fs_out, it_ret.0)?;
            }
        }
        _ => (),
    };

    Ok(())
}

async fn iterate_chronicles<I>(i: I, manifest: Manifest, bundle: Option<Bundle>) -> Result<(Bundle, u64)>
where
    I: accessors::Accessor,
{
    let mut bundle = bundle.unwrap_or(Bundle {
        meta: manifest.meta,
        source: manifest.source,
        strands: HashMap::new(),
        version: None,
    });

    let mut diffs = 0u64;
    let mut iter = i.peekable();

    while let Some(chronicle) = iter.next() {
        let version = chronicle.version;

        for (file_name, v) in chronicle.files {
            let alternator_strand = match alternator::consume(file_name.clone(), v).await {
                Ok(o) => o,
                // If it errors, we'll continue
                // Some prior syntaxes aren't supported by current lexer/parser
                Err(_) => continue,
            };

            match bundle.strands.get_mut(&file_name) {
                // Include file already exist as a strand
                Some(bundle_strand) => {
                    // Iterate all symbols and compare to find differences
                    // Upon difference, update the last_update
                    // If symbol does not exist, update created_at as it was first discovered

                    macro_rules! process_symbol {
                        ($field:ident) => {
                            for (k, v) in alternator_strand.$field {
                                // Attempt to find the equivalent symbol in the bundle

                                match bundle_strand.$field.get_mut(&k) {
                                    Some(b_v) => {
                                        // Symbol does not partialeq, update the updated_at
                                        if v != b_v.symbol {
                                            b_v.last_updated = version.clone();

                                            // Symbol has changed, assign the new value to bundle value
                                            b_v.symbol = v;

                                            diffs += 1;
                                        }
                                    },
                                    None => {
                                        // Symbol is not found in the current bundle, must be new!
                                        bundle_strand.$field.insert(k, Fiber{
                                            symbol: v,
                                            last_updated: version.clone(),
                                            created: version.clone(),
                                        });

                                        diffs += 1;
                                    }
                                }
                            }
                        };
                    }

                    process_symbol!(functions);
                    process_symbol!(methodmaps);
                    process_symbol!(enumstructs);
                    process_symbol!(constants);
                    process_symbol!(defines);
                    process_symbol!(enums);
                    process_symbol!(typesets);
                    process_symbol!(typedefs);
                }
                None => {
                    // This strand (file) does not exist in the bundle
                    // Do a direct insertion and use current version
                    let mut bundle_strand = Strand::default();

                    macro_rules! insert_symbol {
                        ($field:ident) => {
                            for (k, v) in alternator_strand.$field {
                                bundle_strand.$field.insert(
                                    k,
                                    Fiber {
                                        symbol: v,
                                        last_updated: version.clone(),
                                        created: version.clone(),
                                    },
                                );

                                diffs += 1;
                            }
                        };
                    }

                    insert_symbol!(functions);
                    insert_symbol!(methodmaps);
                    insert_symbol!(enumstructs);
                    insert_symbol!(constants);
                    insert_symbol!(defines);
                    insert_symbol!(enums);
                    insert_symbol!(typesets);
                    insert_symbol!(typedefs);

                    bundle.strands.insert(file_name, bundle_strand);
                }
            }
        }

        // If this is the last iteration of chronicles
        // Set the bundle's version to the last commit processed
        if iter.peek().is_none() {
            bundle.version = version.clone();
        }
    }

    Ok((bundle, diffs))
}

fn write_to_disk<T>(loc: &str, t: T) -> Result<()>
where
    T: serde::Serialize,
{
    let s = serde_json::to_string(&t)?;

    std::fs::write(loc, &s)?;

    Ok(())
}
