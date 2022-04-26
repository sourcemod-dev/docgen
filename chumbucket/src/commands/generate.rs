use std::collections::hash_map::{Entry, HashMap};
use std::ops::ShlAssign;

use anyhow::{anyhow, Result};

use clap::ArgMatches;

use schema::bundle::{Bundle, Strand};
use schema::manifest::{Manifest, SourceType};
use schema::metadata::{Metadata, Versioning};
use schema::symbol::{EnumStruct, Enumeration, Metable, MethodMap, TypeSet};

use walker::Walker;

use crate::accessors::{Chronicle, Git};
use crate::utils::write_to_disk;

/// Generate Subcommand
/// Takes independent manifest or include file and convert to Bundle or AlternatorSrand
pub async fn generate_command(matches: &ArgMatches) -> Result<()> {
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
                &manifest
                    .source
                    .repository
                    .clone()
                    .ok_or(anyhow!("Missing source repository"))?,
                &manifest.meta.name,
                manifest
                    .source
                    .patterns
                    .clone()
                    .ok_or(anyhow!("Missing source patterns"))?,
            )?;

            let latest_file_names = walker.latest_file_names()?;
            let git = Git::from_walker(from_time, &mut walker)?;

            let it_ret = iterate_chronicles(git, manifest, bundle, latest_file_names).await?;

            // If there are differences, write to file
            // JSON object keys are not guaranteed to be in ordered each time
            // This check exists to avoid changing of E-tag on CDN proxy
            if it_ret.1 > 0 {
                write_to_disk(fs_out, it_ret.0)?;
            }
        }
        // TODO: Implement direct
        SourceType::Direct => {}
    };

    Ok(())
}

async fn iterate_chronicles<I>(
    i: I,
    manifest: Manifest,
    bundle: Option<Bundle>,
    file_names: Vec<String>,
) -> Result<(Bundle, u64)>
where
    I: Iterator<Item = Chronicle>,
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

            let mut processor =
                ChronicleProcessor::new(&mut bundle, version.clone(), file_name, alternator_strand);
            diffs += processor.process();
        }

        // If this is the last iteration of chronicles
        // Set the bundle's version to the last commit processed
        if iter.peek().is_none() {
            bundle.version = version.clone();
        }
    }

    // If strand is not in the file_names, remove it
    bundle.strands.retain(|key, _| file_names.contains(key));

    Ok((bundle, diffs))
}

struct ChronicleProcessor<'b> {
    strand: &'b mut Strand,

    dp_strand: Strand,

    version: Option<Versioning>,

    diffs: u64,
}

impl<'b> ChronicleProcessor<'b> {
    pub fn new(
        bundle: &'b mut Bundle,
        version: Option<Versioning>,
        key: String,
        dp_strand: Strand,
    ) -> Self {
        let mut diffs = 0;

        // Check if the strand exists in the bundle
        if let Entry::Vacant(entry) = bundle.strands.entry(key.clone()) {
            entry.insert(dp_strand.clone());

            diffs += 1;
        }

        Self {
            strand: bundle.strands.get_mut(&key).unwrap(),
            version,
            dp_strand,
            diffs,
        }
    }

    pub fn process(&mut self) -> u64 {
        // Functions
        self.diffs += Self::process_fibers(
            &mut self.strand.functions,
            &self.dp_strand.functions,
            self.version.clone(),
        );
        // Constants
        self.diffs += Self::process_fibers(
            &mut self.strand.constants,
            &self.dp_strand.constants,
            self.version.clone(),
        );
        // Defines
        self.diffs += Self::process_fibers(
            &mut self.strand.defines,
            &self.dp_strand.defines,
            self.version.clone(),
        );
        // Methodmaps
        self.diffs += Self::process_methodmaps(
            &mut self.strand.methodmaps,
            &self.dp_strand.methodmaps,
            self.version.clone(),
        );
        // Enumstructs
        self.diffs += Self::process_enumstructs(
            &mut self.strand.enumstructs,
            &self.dp_strand.enumstructs,
            self.version.clone(),
        );
        // Enums
        self.diffs += Self::process_enums(
            &mut self.strand.enums,
            &self.dp_strand.enums,
            self.version.clone(),
        );
        // Typesets
        self.diffs += Self::process_typesets(
            &mut self.strand.typesets,
            &self.dp_strand.typesets,
            self.version.clone(),
        );
        // Type definitions
        self.diffs += Self::process_fibers(
            &mut self.strand.typedefs,
            &self.dp_strand.typedefs,
            self.version.clone(),
        );

        self.diffs
    }

    fn process_methodmaps(
        existing: &mut HashMap<String, MethodMap>,
        new: &HashMap<String, MethodMap>,
        version: Option<Versioning>,
    ) -> u64 {
        let mut diff = 0;

        diff += Self::process_fibers(existing, new, version.clone());

        for (k, v) in existing {
            // Process methods
            diff += Self::process_fibers(
                &mut v.methods,
                &new.get(k).unwrap().methods,
                version.clone(),
            );

            // Process properties
            diff += Self::process_fibers(
                &mut v.properties,
                &new.get(k).unwrap().properties,
                version.clone(),
            );
        }

        diff
    }

    fn process_enumstructs(
        existing: &mut HashMap<String, EnumStruct>,
        new: &HashMap<String, EnumStruct>,
        version: Option<Versioning>,
    ) -> u64 {
        let mut diff = 0;

        diff += Self::process_fibers(existing, new, version.clone());

        for (k, v) in existing {
            // Process methods
            diff += Self::process_fibers(
                &mut v.methods,
                &new.get(k).unwrap().methods,
                version.clone(),
            );

            // Process fields
            diff +=
                Self::process_fibers(&mut v.fields, &new.get(k).unwrap().fields, version.clone());
        }

        diff
    }

    fn process_enums(
        existing: &mut HashMap<String, Enumeration>,
        new: &HashMap<String, Enumeration>,
        version: Option<Versioning>,
    ) -> u64 {
        let mut diff = 0;

        diff += Self::process_fibers(existing, new, version.clone());

        for (k, v) in existing {
            // Process entries
            diff += Self::process_fibers(
                &mut v.entries,
                &new.get(k).unwrap().entries,
                version.clone(),
            );
        }

        diff
    }

    fn process_typesets(
        existing: &mut HashMap<String, TypeSet>,
        new: &HashMap<String, TypeSet>,
        version: Option<Versioning>,
    ) -> u64 {
        let mut diff = 0;

        diff += Self::process_fibers(existing, new, version.clone());

        for (k, v) in existing {
            // Process types
            diff += Self::process_fibers(&mut v.types, &new.get(k).unwrap().types, version.clone());
        }

        diff
    }

    fn process_fibers<T: Metable + Clone + ShlAssign + PartialEq>(
        existing: &mut HashMap<String, T>,
        new: &HashMap<String, T>,
        version: Option<Versioning>,
    ) -> u64 {
        let mut diff = 0;

        // Handle any new entries
        for (k, v) in new {
            if let Entry::Vacant(entry) = existing.entry(k.clone()) {
                entry.insert(v.clone());
                diff += 1;
            }
        }

        let mut to_remove = Vec::new();

        // Handle any deleted entries
        for (k, _) in existing.iter() {
            if !new.contains_key(k) {
                to_remove.push(k.clone());
            }
        }

        diff += to_remove.len() as u64;

        for k in to_remove {
            existing.remove(&k);
        }

        // At this point, both list should have the same keys, so we can safely iterate over them
        // Handle any updated entries
        for (k, v) in existing.iter_mut() {
            let dp_v = new.get(k).unwrap();

            let is_diff = v != dp_v;

            if is_diff {
                *v <<= dp_v.clone();
                diff += 1;
            }

            // Update the metadata
            match v.metadata() {
                Some(m) if is_diff => {
                    m.last_updated = version.clone();
                }
                None if !is_diff => {
                    *v.metadata() = Some(Metadata {
                        last_updated: version.clone(),
                        created: version.clone(),
                    });
                }
                _ => {}
            }
        }

        diff
    }
}
