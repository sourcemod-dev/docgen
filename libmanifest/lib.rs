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

    pub files: Files,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Files {
    pub plugins: Option<Vec<String>>,

    pub includes: Option<Vec<String>>,

    pub translations: Option<Vec<String>>,

    pub configs: Option<Vec<String>>,

    pub gamedata: Option<Vec<String>>,
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
