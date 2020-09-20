use std::path::Path;

use git2::{Delta, IntoCString, Oid, Pathspec, PathspecFlags, Repository, Sort};

mod error;

use error::Result;

pub struct Walker {
    repo: Repository,

    pathspec: Pathspec,

    spec_diffs: Vec<CommitDiffs>,
}

#[derive(Debug)]
pub struct CommitDiffs {
    pub commit: Oid,

    pub stem_diffs: Vec<String>,
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
            spec_diffs: Vec::new(),
        })
    }

    pub fn spec_diffs(&self) -> &Vec<CommitDiffs> {
        &self.spec_diffs
    }

    pub fn walk(&mut self, from: Option<Oid>) -> Result<()> {
        let mut revwalk = self.repo.revwalk()?;

        revwalk.set_sorting(Sort::TIME | Sort::REVERSE)?;

        match from {
            Some(v) => revwalk.push(v)?,
            None => revwalk.push_head()?,
        }

        for oid in revwalk {
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

                    let diff_stems: Vec<String> = ml
                        .diff_entries()
                        .filter(|v| v.status() != Delta::Deleted)
                        .map(|v| v.new_file().path())
                        .filter(|v| v.is_some())
                        .map(|v| v.unwrap().file_stem())
                        .filter(|v| v.is_some())
                        .map(|v| v.unwrap().to_string_lossy().into_owned())
                        .collect();

                    if !diff_stems.is_empty() {
                        self.spec_diffs.push(CommitDiffs {
                            commit: commit.id(),
                            stem_diffs: diff_stems,
                        });
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
