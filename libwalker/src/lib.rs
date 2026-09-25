use std::collections::HashSet;
use std::ops::Range;
use std::path::{Path, PathBuf};

use git2::{Commit, Delta, IntoCString, Oid, Pathspec, PathspecFlags, Repository, Sort, Tree};

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
        let mut on_chain = HashSet::new();
        let mut reached_checkpoint = false;
        let mut next = Some(self.repo.head()?.peel_to_commit()?);

        while let Some(commit) = next {
            if Some(commit.id()) == checkpoint {
                reached_checkpoint = true;
                break;
            }

            if !on_chain.insert(commit.id()) {
                break;
            }

            next = match commit.parent_count() {
                0 => self.find_predecessor(&commit, &on_chain)?,
                _ => Some(commit.parent(0)?),
            };

            chain.push(commit);
        }

        chain.reverse();

        let mut spec_diffs = Vec::new();
        let mut prev: Option<Commit> = None;
        let mut count = 0;

        for commit in chain {
            let parents: Vec<Oid> = commit.parent_ids().collect();

            count = match (parents.split_first(), &prev) {
                // A merge also brings in every commit of the merged branch not already in the first parent
                (Some((first, rest)), Some(prev)) if *first == prev.id() => {
                    count
                        + 1
                        + match rest.is_empty() {
                            true => 0,
                            false => self.rev_list_count(rest, Some(*first))?,
                        }
                }
                // Start of the walk, or a root continuing from its predecessor
                _ => self.rev_list_count(&[commit.id()], None)?,
            };

            // If the checkpoint couldn't be found, fall back to skipping commits older than the time
            let skip = match (reached_checkpoint, since_time) {
                (false, Some(from_time)) => commit.time().seconds() < from_time,
                _ => false,
            };

            if !skip {
                // Compared against the previous commit walked (or the first parent at the start of
                // the walk), so a root without a predecessor is compared against an empty tree
                let base_tree = match (&prev, parents.first()) {
                    (Some(prev), _) => Some(prev.tree()?),
                    (None, Some(_)) => Some(commit.parent(0)?.tree()?),
                    (None, None) => None,
                };

                let diff =
                    self.repo
                        .diff_tree_to_tree(base_tree.as_ref(), Some(&commit.tree()?), None)?;

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

            prev = Some(commit);
        }

        Ok(DiffList {
            range: 0..spec_diffs.len(),
            spec_diffs,
            walker: self,
        })
    }

    /// A root commit may re-import the files of an older, otherwise disconnected history, such as
    /// a VCS conversion restarting from a snapshot. Returns the latest commit reachable from HEAD,
    /// no newer than the root and not already walked, whose matching files are identical to the
    /// root's, so the walk can continue through that history
    fn find_predecessor(
        &self,
        root: &Commit,
        on_chain: &HashSet<Oid>,
    ) -> Result<Option<Commit<'_>>> {
        let files = self.matching_files(&root.tree()?)?;

        if files.is_empty() {
            return Ok(None);
        }

        let mut revwalk = self.repo.revwalk()?;

        revwalk.set_sorting(Sort::TIME)?;

        revwalk.push_head()?;

        for oid in revwalk {
            let oid = oid?;

            if on_chain.contains(&oid) {
                continue;
            }

            let commit = self.repo.find_commit(oid)?;

            if commit.time().seconds() > root.time().seconds() {
                continue;
            }

            if self.matching_files(&commit.tree()?)? == files {
                return Ok(Some(commit));
            }
        }

        Ok(None)
    }

    /// Paths matching the pathspec in a tree, with their blob ids
    fn matching_files(&self, tree: &Tree) -> Result<Vec<(PathBuf, Oid)>> {
        let ml = self.pathspec.match_tree(tree, PathspecFlags::DEFAULT)?;

        let mut files = Vec::new();

        for entry in ml.entries() {
            let path = PathBuf::from(String::from_utf8_lossy(entry).into_owned());
            let id = tree.get_path(&path)?.id();

            files.push((path, id));
        }

        files.sort();

        Ok(files)
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
