use std::collections::HashMap;

use anyhow::Result;

use schema::metadata::Versioning;

use walker::{DiffList, Walker};

use super::Chronicle;

pub struct Git<'g>(DiffList<'g>);

impl<'g> Git<'g> {
    pub fn from_walker(since: Option<&Versioning>, walker: &'g mut Walker) -> Result<Self> {
        Ok(Self(walker.walk(
            since.map(|v| v.hash.as_str()),
            since.map(|v| v.time),
        )?))
    }
}

impl<'g> Iterator for Git<'g> {
    type Item = Chronicle;

    fn next(&mut self) -> Option<Self::Item> {
        let blob_contents = self.0.next()?;

        let version = {
            let bc = blob_contents.first()?;

            Versioning {
                hash: bc.commit.to_string(),
                count: bc.count,
                time: bc.time,
            }
        };

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
