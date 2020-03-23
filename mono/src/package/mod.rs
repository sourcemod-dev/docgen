pub struct Package {
    /// Package identifier that is assumed to be unique
    pub name: String,

    /// Package description
    /// 
    /// Used to display information when selecting third-party includes
    /// Auto-populated with README if found
    pub description: Option<String>,

    /// Version string that is compliant with semver
    /// 
    /// Latest release of SVC commit release will be used if this is not present
    pub version: Option<String>,

    /// Author of the package
    pub author: Option<String>,

    /// Git, SVN, etc + expr globs
    pub repository: Option<Repository>,

    /// Non SVC package endpoint + expr globs
    pub files: Option<Files>,
}

pub struct Repository {
    pub r#type: String,

    pub url: String,
}

pub struct Files {
    pub url: String,

    pub patterns: Vec<String>,
}