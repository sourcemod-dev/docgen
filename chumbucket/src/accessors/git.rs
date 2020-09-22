use super::{Accessor, Chronicle};

use anyhow::Result;
use schema::manifest::Manifest;
use std::collections::HashMap;
use walker::{DiffList, Walker};

pub struct Git<'g>(DiffList<'g>);

impl<'g> Git<'g> {
    pub fn from_walker(
        manifest: &Manifest,
        from: Option<&str>,
        walker: &'g mut Walker,
    ) -> Result<Self> {
        Ok(Self(walker.walk(from)?))
    }
}

impl<'g> Iterator for Git<'g> {
    type Item = Chronicle;

    fn next(&mut self) -> Option<Self::Item> {
        let blob_contents = self.0.next()?;

        let version = blob_contents.first()?.commit.to_string();
        let mut files: HashMap<String, Vec<u8>> = HashMap::new();

        for blob in blob_contents {
            files.insert(
                blob.path.file_stem()?.to_string_lossy().to_string(),
                blob.content,
            );
        }

        Some(Chronicle {
            version: Some(version),
            files,
        })
    }
}
