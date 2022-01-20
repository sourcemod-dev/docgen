use std::collections::HashMap;

use schema::metadata::Versioning;

mod git;

pub use self::git::Git;

pub struct Chronicle {
    /// The version this chronicle represents
    pub version: Option<Versioning>,

    /// List of files that has been modified at this version
    pub files: HashMap<String, Vec<u8>>,
}

/// Accessor returns an iterator that pulls from oldest to newest
pub trait Accessor = Iterator<Item = Chronicle>;
