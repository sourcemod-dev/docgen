use std::ops::Range;
use std::path::{Path, PathBuf};

use git2::{Delta, IntoCString, Oid, Pathspec, PathspecFlags, Repository};

mod error;

use error::{Result, WalkerError};

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

    pub count: u64,

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

    /// Walks the first-parent history of HEAD, oldest first, collecting the commits that touched
    /// any path matching the pathspec.
    ///
    /// `since_commit` is the last commit processed by a previous walk; only commits after it are
    /// collected. If it isn't on the current history, commits older than `since_time` are skipped
    /// instead.
    pub fn walk(
        &mut self,
        since_commit: Option<&str>,
        since_time: Option<i64>,
    ) -> Result<DiffList<'_>> {
        let checkpoint = since_commit.and_then(|v| Oid::from_str(v).ok());

        // Follow first parents only, so commits are visited in the order they landed on the branch.
        // Commits from merged branches are observed through their merge commit, otherwise their
        // contents would be interleaved with, and reverted by, the mainline commits around them
        let mut chain = Vec::new();
        let mut reached_checkpoint = false;
        let mut next = Some(self.repo.head()?.peel_to_commit()?);

        while let Some(commit) = next {
            if Some(commit.id()) == checkpoint {
                reached_checkpoint = true;
                break;
            }

            next = match commit.parent_count() {
                0 => None,
                _ => Some(commit.parent(0)?),
            };

            chain.push(commit);
        }

        chain.reverse();

        // Rev-list count of the commit the chain starts from
        let mut count = match chain.first().map(|v| v.parent_ids().next()) {
            Some(Some(parent)) => self.rev_list_count(&[parent], None)?,
            _ => 0,
        };

        let mut spec_diffs = Vec::new();

        for commit in chain {
            let parents: Vec<Oid> = commit.parent_ids().collect();

            // A merge also brings in every commit of the merged branch not already in the first parent
            count += 1 + match parents.split_first() {
                Some((first, rest)) if !rest.is_empty() => {
                    self.rev_list_count(rest, Some(*first))?
                }
                _ => 0,
            };

            // If the checkpoint couldn't be found, fall back to skipping commits older than the time
            if !reached_checkpoint {
                if let Some(from_time) = since_time {
                    if commit.time().seconds() < from_time {
                        continue;
                    }
                }
            }

            // The root commit is compared against an empty tree, so files it adds are picked up
            let parent_tree = match parents.first() {
                Some(_) => Some(commit.parent(0)?.tree()?),
                None => None,
            };

            let diff =
                self.repo
                    .diff_tree_to_tree(parent_tree.as_ref(), Some(&commit.tree()?), None)?;

            let ml = self.pathspec.match_diff(&diff, PathspecFlags::DEFAULT)?;

            let diff_stems: Vec<PathBuf> = ml
                .diff_entries()
                .filter(|v| v.status() != Delta::Deleted)
                .filter_map(|v| v.new_file().path())
                .map(|v| v.to_path_buf())
                .collect();

            if !diff_stems.is_empty() {
                spec_diffs.push(CommitDiffs {
                    commit: commit.id(),
                    count,
                    path_diffs: diff_stems,
                });
            }
        }

        Ok(DiffList {
            range: 0..spec_diffs.len(),
            spec_diffs,
            walker: self,
        })
    }

    /// Number of commits reachable from `from`, excluding those reachable from `hide`
    fn rev_list_count(&self, from: &[Oid], hide: Option<Oid>) -> Result<u64> {
        let mut revwalk = self.repo.revwalk()?;

        for oid in from {
            revwalk.push(*oid)?;
        }

        if let Some(oid) = hide {
            revwalk.hide(oid)?;
        }

        let mut count = 0;

        for oid in revwalk {
            oid?;
            count += 1;
        }

        Ok(count)
    }

    pub fn latest_file_names(&mut self) -> Result<Vec<String>> {
        let mut file_names = Vec::new();

        let tree = self.repo.find_reference("HEAD")?.peel_to_tree()?;

        let pathspec_entries = self.pathspec.match_tree(&tree, PathspecFlags::DEFAULT)?;

        for entry in pathspec_entries.entries() {
            let lossy_fmt = String::from_utf8_lossy(entry).into_owned();
            let file_name = Path::new(&lossy_fmt)
                .file_stem()
                .ok_or(WalkerError::InvalidPath)?
                .to_string_lossy()
                .into_owned();

            file_names.push(file_name);
        }

        Ok(file_names)
    }
}

pub struct BlobContent {
    pub commit: Oid,

    pub count: u64,

    pub time: i64,

    pub path: PathBuf,

    pub content: Vec<u8>,
}

impl<'w> Iterator for DiffList<'w> {
    type Item = Vec<BlobContent>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let bcs = self.next_contents()?;

            // Keep going if every entry of this commit was skipped
            if !bcs.is_empty() {
                return Some(bcs);
            }
        }
    }
}

impl<'w> DiffList<'w> {
    fn next_contents(&mut self) -> Option<Vec<BlobContent>> {
        let spec_diff = self.range.next().and_then(|i| self.spec_diffs.get(i))?;

        let commit = self.walker.repo.find_commit(spec_diff.commit).ok()?;

        let tree = commit.tree().ok()?;

        let mut bcs = Vec::new();

        for path in &spec_diff.path_diffs {
            // Skip entries that aren't readable blobs (submodules, etc.) rather than ending the walk
            let content = match tree
                .get_path(path)
                .and_then(|v| v.to_object(&self.walker.repo))
            {
                Ok(obj) => match obj.as_blob() {
                    Some(blob) => blob.content().to_owned(),
                    None => continue,
                },
                Err(_) => continue,
            };

            bcs.push(BlobContent {
                commit: spec_diff.commit,
                count: spec_diff.count,
                time: commit.time().seconds(),
                path: path.to_owned(),
                content,
            })
        }

        Some(bcs)
    }
}
