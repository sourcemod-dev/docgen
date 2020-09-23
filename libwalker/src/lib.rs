use std::ops::Range;
use std::path::{Path, PathBuf};

use git2::{Delta, IntoCString, Oid, Pathspec, PathspecFlags, Repository, Sort};

mod error;

use error::Result;

pub struct Walker {
    repo: Repository,

    pathspec: Pathspec,
}

pub struct DiffList<'w> {
    spec_diffs: Vec<CommitDiffs>,

    range: Range<usize>,

    walker: &'w Walker,
}

pub struct CommitDiffs {
    pub commit: Oid,

    pub path_diffs: Vec<PathBuf>,
}

impl Walker {
    pub fn new<R, T, P>(repo: R, path_specs: P) -> Result<Self>
    where
        R: AsRef<Path>,
        T: IntoCString,
        P: IntoIterator<Item = T>,
    {
        Ok(Self {
            repo: Repository::open(repo)?,
            pathspec: Pathspec::new(path_specs)?,
        })
    }

    pub fn from_remote<P, T, S>(url: &str, into: P, path_specs: S) -> Result<Self>
    where
        P: AsRef<Path>,
        T: IntoCString,
        S: IntoIterator<Item = T>,
    {
        // If repository exists locally, open instead
        let repo = match into.as_ref().exists() {
            true => Repository::open(into)?,
            _ => Repository::clone(url, into)?,
        };

        Ok(Self {
            repo,
            pathspec: Pathspec::new(path_specs)?,
        })
    }

    pub fn walk(&mut self, from: Option<&str>) -> Result<DiffList> {
        let mut revwalk = self.repo.revwalk()?;

        revwalk.set_sorting(Sort::TIME | Sort::REVERSE)?;

        match from {
            Some(v) => revwalk.push(Oid::from_str(v)?)?,
            None => revwalk.push_head()?,
        }

        let mut spec_diffs = Vec::new();

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

                    let diff_stems: Vec<PathBuf> = ml
                        .diff_entries()
                        .filter(|v| v.status() != Delta::Deleted)
                        .map(|v| v.new_file().path())
                        .filter(|v| v.is_some())
                        .map(|v| v.unwrap().to_path_buf())
                        .collect();

                    if !diff_stems.is_empty() {
                        spec_diffs.push(CommitDiffs {
                            commit: commit.id(),
                            path_diffs: diff_stems,
                        });
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        Ok(DiffList {
            range: 0..spec_diffs.len(),
            spec_diffs,
            walker: self,
        })
    }
}

pub struct BlobContent {
    pub commit: Oid,

    pub time: i64,

    pub path: PathBuf,

    pub content: Vec<u8>,
}

impl<'w> Iterator for DiffList<'w> {
    type Item = Vec<BlobContent>;

    fn next(&mut self) -> Option<Self::Item> {
        let spec_diff = self.range.next().and_then(|i| self.spec_diffs.get(i))?;

        let commit = self.walker.repo.find_commit(spec_diff.commit).ok()?;

        let tree = commit.tree().ok()?;

        let mut bcs = Vec::new();

        for path in &spec_diff.path_diffs {
            let te = tree.get_path(&path).ok()?;

            let obj = te.to_object(&self.walker.repo).ok()?;

            let content = obj.as_blob()?.content().to_owned();

            bcs.push(BlobContent {
                commit: spec_diff.commit,
                time: commit.time().seconds(),
                path: path.to_owned(),
                content,
            })
        }

        Some(bcs)
    }
}
