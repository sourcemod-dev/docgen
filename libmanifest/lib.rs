use std::collections::BTreeMap;

use serde::{
    Serialize,
    Deserialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub package: Package,

    pub dependencies: BTreeMap<String, String>,

    pub files: SMBaseFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    Exact(String),
    Pattern(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SMBaseFiles {
    pub configs: Option<Vec<MatchType>>,

    pub data: Option<Vec<MatchType>>,

    pub extensions: Option<Vec<MatchType>>,

    pub gamedata: Option<Vec<MatchType>>,

    pub plugins: Option<Vec<MatchType>>,

    pub scripting: Option<Vec<MatchType>>,

    pub translations:Option<Vec<MatchType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Package {
    pub name: String,

    pub version: String,

    pub authors: Vec<String>,

    pub description: Option<String>,

    pub homepage: Option<String>,

    pub documentation: Option<String>,

    pub keywords: Vec<String>,
}
