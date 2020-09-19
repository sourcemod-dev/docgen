use std::collections::HashMap;
use std::path::Path;

use git2::{IntoCString, Oid, Pathspec, PathspecFlags, Repository, Sort};

mod error;

use error::Result;

pub struct Walker {
    repo: Repository,

    pathspec: Pathspec,

    path_commits: HashMap<String, Vec<Oid>>,

    commits: Vec<String>,
}

impl Walker {
    pub fn new<R, T, P>(repo: R, paths: P) -> Result<Self>
    where
        R: AsRef<Path>,
        T: IntoCString,
        P: IntoIterator<Item = T>,
    {
        Ok(Self {
            repo: Repository::open(repo)?,
            pathspec: Pathspec::new(paths)?,
            path_commits: HashMap::new(),
            commits: Vec::new(),
        })
    }

    pub fn commits(&self) -> Vec<String> {
        self.commits.clone()
    }

    pub fn walk(&mut self, from: Option<Oid>) -> Result<()> {
        let mut revwalk = self.repo.revwalk()?;

        revwalk.set_sorting(Sort::TIME | Sort::REVERSE)?;

        match from {
            Some(v) => revwalk.push(v)?,
            None => revwalk.push_head()?,
        }

        while let Some(oid) = revwalk.next() {
            let oid = oid?;

            let commit = self.repo.find_commit(oid)?;

            let c_tree = commit.tree()?;

            let parent_count = commit.parent_count();

            match parent_count {
                c if c == 1 => {
                    let parent = commit.parent(0)?;

                    let diff =
                        self.repo
                            .diff_tree_to_tree(Some(&parent.tree()?), Some(&c_tree), None)?;

                    let ml = self.pathspec.match_diff(&diff, PathspecFlags::DEFAULT)?;

                    for v in ml.diff_entries() {
                        if let Some(new_file) = v.new_file().path() {
                            if let Some(file) = new_file.to_str() {
                                match self.path_commits.get_mut(file) {
                                    Some(vec) => vec.push(commit.id()),
                                    None => {
                                        self.path_commits
                                            .insert(file.to_string(), vec![commit.id()]);
                                    }
                                }
                            }
                        }
                    }

                    if ml.diff_entries().len() > 0 {
                        self.commits.push(commit.id().to_string());
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        Ok(())
    }
}
