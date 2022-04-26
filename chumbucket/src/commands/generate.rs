use std::collections::hash_map::{Entry, HashMap};

use anyhow::{anyhow, Result};

use clap::ArgMatches;

use schema::{
    bundle::{Bundle, Strand},
    manifest::{Manifest, SourceType},
    metadata::{Metadata, Versioning},
    symbol::{
        Constant, Define, Entry as EnumEntry, EnumStruct, Enumeration, Field, Function, MethodMap,
        Property, Type, TypeDefinition, TypeSet,
    },
};

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

            let git = Git::from_walker(from_time, &mut walker)?;

            let it_ret = iterate_chronicles(git, manifest, bundle).await?;

            // If there are differences, write to file
            // JSON object keys are not guaranteed to be in ordered each time
            // This check exists to avoid changing of E-tag on CDN proxy
            if it_ret.1 > 0 {
                write_to_disk(fs_out, it_ret.0)?;
            }
        }
        // TODO: Implement direct
        _ => (),
    };

    Ok(())
}

async fn iterate_chronicles<I>(
    i: I,
    manifest: Manifest,
    bundle: Option<Bundle>,
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
        let functions = self.dp_strand.functions.clone();

        for (k, v) in &functions {
            if let Entry::Vacant(entry) = self.strand.functions.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut function = self.strand.functions.remove(k).unwrap();

            self.process_function(&mut function, v.clone());

            self.strand.functions.insert(k.to_string(), function);
        }

        for (k, v) in &self.dp_strand.methodmaps.clone() {
            if let Entry::Vacant(entry) = self.strand.methodmaps.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut mm = self.strand.methodmaps.remove(k).unwrap();

            self.process_methodmap(&mut mm, v.clone());

            self.strand.methodmaps.insert(k.to_string(), mm);
        }

        for (k, v) in &self.dp_strand.enumstructs.clone() {
            if let Entry::Vacant(entry) = self.strand.enumstructs.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut es = self.strand.enumstructs.remove(k).unwrap();

            self.process_enumstruct(&mut es, v.clone());

            self.strand.enumstructs.insert(k.to_string(), es);
        }

        for (k, v) in &self.dp_strand.constants.clone() {
            if let Entry::Vacant(entry) = self.strand.constants.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut c = self.strand.constants.remove(k).unwrap();

            self.process_constant(&mut c, v.clone());

            self.strand.constants.insert(k.to_string(), c);
        }

        for (k, v) in &self.dp_strand.defines.clone() {
            if let Entry::Vacant(entry) = self.strand.defines.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut d = self.strand.defines.remove(k).unwrap();

            self.process_define(&mut d, v.clone());

            self.strand.defines.insert(k.to_string(), d);
        }

        for (k, v) in &self.dp_strand.enums.clone() {
            if let Entry::Vacant(entry) = self.strand.enums.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut e = self.strand.enums.remove(k).unwrap();

            self.process_enum(&mut e, v.clone());

            self.strand.enums.insert(k.to_string(), e);
        }

        for (k, v) in &self.dp_strand.typesets.clone() {
            if let Entry::Vacant(entry) = self.strand.typesets.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut ts = self.strand.typesets.remove(k).unwrap();

            self.process_typeset(&mut ts, v.clone());

            self.strand.typesets.insert(k.to_string(), ts);
        }

        for (k, v) in &self.dp_strand.typedefs.clone() {
            if let Entry::Vacant(entry) = self.strand.typedefs.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut td = self.strand.typedefs.remove(k).unwrap();

            self.process_typedef(&mut td, v.clone());

            self.strand.typedefs.insert(k.to_string(), td);
        }

        self.diffs
    }

    fn process_function(&mut self, function: &mut Function, dp_function: Function) {
        if *function != dp_function {
            *function <<= dp_function;

            self.update_metadata(function.declaration.metadata(), true);
        } else {
            self.update_metadata(function.declaration.metadata(), false);
        }
    }

    fn process_methodmap(&mut self, methodmap: &mut MethodMap, dp_methodmap: MethodMap) {
        if *methodmap != dp_methodmap {
            *methodmap <<= dp_methodmap.clone();

            self.update_metadata(methodmap.declaration.metadata(), true);
        } else {
            self.update_metadata(methodmap.declaration.metadata(), false);
        }

        let methods = dp_methodmap.methods.clone();

        for (k, v) in &methods {
            if let Entry::Vacant(entry) = methodmap.methods.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut method = methodmap.methods.remove(k).unwrap();

            self.process_function(&mut method, v.clone());

            methodmap.methods.insert(k.to_string(), method);
        }

        let properties = dp_methodmap.properties.clone();

        for (k, v) in &properties {
            if let Entry::Vacant(entry) = methodmap.properties.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut property = methodmap.properties.remove(k).unwrap();

            self.process_property(&mut property, v.clone());

            methodmap.properties.insert(k.to_string(), property);
        }
    }

    fn process_enumstruct(&mut self, enumstruct: &mut EnumStruct, dp_enumstruct: EnumStruct) {
        if *enumstruct != dp_enumstruct {
            *enumstruct <<= dp_enumstruct.clone();

            self.update_metadata(enumstruct.declaration.metadata(), true);
        } else {
            self.update_metadata(enumstruct.declaration.metadata(), false);
        }

        let methods = dp_enumstruct.methods.clone();

        for (k, v) in &methods {
            if let Entry::Vacant(entry) = enumstruct.methods.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut method = enumstruct.methods.remove(k).unwrap();

            self.process_function(&mut method, v.clone());

            enumstruct.methods.insert(k.to_string(), method);
        }

        let fields = dp_enumstruct.fields.clone();

        for (k, v) in &fields {
            if let Entry::Vacant(entry) = enumstruct.fields.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut field = enumstruct.fields.remove(k).unwrap();

            self.process_field(&mut field, v.clone());

            enumstruct.fields.insert(k.to_string(), field);
        }
    }

    fn process_constant(&mut self, constant: &mut Constant, dp_constant: Constant) {
        if *constant != dp_constant {
            *constant <<= dp_constant;

            self.update_metadata(constant.declaration.metadata(), true);
        } else {
            self.update_metadata(constant.declaration.metadata(), false);
        }
    }

    fn process_define(&mut self, define: &mut Define, dp_define: Define) {
        if *define != dp_define {
            *define <<= dp_define;

            self.update_metadata(define.declaration.metadata(), true);
        } else {
            self.update_metadata(define.declaration.metadata(), false);
        }
    }

    fn process_enum(&mut self, enum_: &mut Enumeration, dp_enum: Enumeration) {
        if *enum_ != dp_enum {
            *enum_ <<= dp_enum.clone();

            self.update_metadata(enum_.declaration.metadata(), true);
        } else {
            self.update_metadata(enum_.declaration.metadata(), false);
        }

        let entries = dp_enum.entries.clone();

        for (k, v) in &entries {
            if let Entry::Vacant(entry) = enum_.entries.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut entry = enum_.entries.remove(k).unwrap();

            self.process_enum_entry(&mut entry, v.clone());

            enum_.entries.insert(k.to_string(), entry);
        }
    }

    fn process_typeset(&mut self, typeset: &mut TypeSet, dp_typeset: TypeSet) {
        if *typeset != dp_typeset {
            *typeset <<= dp_typeset.clone();

            self.update_metadata(typeset.declaration.metadata(), true);
        } else {
            self.update_metadata(typeset.declaration.metadata(), false);
        }

        let types = dp_typeset.types.clone();

        for (k, v) in &types {
            if let Entry::Vacant(entry) = typeset.types.entry(k.clone()) {
                entry.insert(v.clone());

                self.diffs += 1;
            }

            let mut type_ = typeset.types.remove(k).unwrap();

            self.process_type(&mut type_, v.clone());

            typeset.types.insert(k.to_string(), type_);
        }
    }

    fn process_typedef(&mut self, typedef: &mut TypeDefinition, dp_typedef: TypeDefinition) {
        if *typedef != dp_typedef {
            *typedef <<= dp_typedef.clone();

            self.update_metadata(typedef.declaration.metadata(), true);
        } else {
            self.update_metadata(typedef.declaration.metadata(), false);
        }
    }

    fn process_property(&mut self, property: &mut Property, dp_property: Property) {
        if *property != dp_property {
            *property <<= dp_property;

            self.update_metadata(property.declaration.metadata(), true);
        } else {
            self.update_metadata(property.declaration.metadata(), false);
        }
    }

    fn process_field(&mut self, field: &mut Field, dp_field: Field) {
        if *field != dp_field {
            *field <<= dp_field;

            self.update_metadata(field.declaration.metadata(), true);
        } else {
            self.update_metadata(field.declaration.metadata(), false);
        }
    }

    fn process_enum_entry(&mut self, entry: &mut EnumEntry, dp_entry: EnumEntry) {
        if *entry != dp_entry {
            *entry <<= dp_entry;

            self.update_metadata(entry.declaration.metadata(), true);
        } else {
            self.update_metadata(entry.declaration.metadata(), false);
        }
    }

    fn process_type(&mut self, type_: &mut Type, dp_type: Type) {
        if *type_ != dp_type {
            *type_ <<= dp_type.clone();

            self.update_metadata(&mut type_.documentation.metadata, true);
        } else {
            self.update_metadata(&mut type_.documentation.metadata, false);
        }
    }

    fn update_metadata(&self, m: &mut Option<Metadata>, diff: bool) {
        let t = if let Some(a) = self.version.clone() {
            a
        } else {
            return;
        };

        match m {
            Some(a) if diff => {
                a.last_updated = Some(t);
            }
            None if !diff => {
                *m = Some(Metadata {
                    last_updated: Some(t.clone()),
                    created: Some(t.clone()),
                });
            }
            _ => {}
        }
    }
}
