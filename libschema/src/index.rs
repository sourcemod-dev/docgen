use serde::Serialize;

use super::meta::Meta;
use super::manifest::Source;

pub type Indices = Vec<Index>;

#[derive(Serialize)]
pub struct Index {
    /// Meta descriptor of manifest content
    pub meta: Meta,

    /// Meta content source
    pub source: Source,
}
