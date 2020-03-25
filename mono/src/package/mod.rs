mod general;
mod repository;
mod files;

use general::General;
use repository::Repository;
use files::Files;

pub struct Package {
    pub general: General,

    /// Git, SVN, etc + expr globs
    pub repository: Option<Repository>,

    /// Non SVC package endpoint + expr globs
    pub files: Option<Files>,
}
