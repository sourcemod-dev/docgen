extern crate walker;

use std::path::{Path, PathBuf};

use git2::{Commit, Oid, Repository, Signature, Time};

use walker::Walker;

struct Fixture {
    path: PathBuf,
    repo: Repository,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("walker-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&path);

        Self {
            repo: Repository::init(&path).unwrap(),
            path,
        }
    }

    /// Commits `files` on top of `parents`' first tree, without moving any reference
    fn commit(&self, time: i64, parents: &[Oid], files: &[(&str, &str)]) -> Oid {
        let parents: Vec<Commit> = parents
            .iter()
            .map(|v| self.repo.find_commit(*v).unwrap())
            .collect();

        let mut index = git2::Index::new().unwrap();

        if let Some(parent) = parents.first() {
            index.read_tree(&parent.tree().unwrap()).unwrap();
        }

        for (path, content) in files {
            let blob = self.repo.blob(content.as_bytes()).unwrap();
            let mut entry = git2::IndexEntry {
                ctime: git2::IndexTime::new(0, 0),
                mtime: git2::IndexTime::new(0, 0),
                dev: 0,
                ino: 0,
                mode: 0o100644,
                uid: 0,
                gid: 0,
                file_size: content.len() as u32,
                id: blob,
                flags: 0,
                flags_extended: 0,
                path: path.as_bytes().to_vec(),
            };
            entry.flags = entry.path.len() as u16;
            index.add(&entry).unwrap();
        }

        let tree = self
            .repo
            .find_tree(index.write_tree_to(&self.repo).unwrap())
            .unwrap();
        let sig = Signature::new("test", "test@example.com", &Time::new(time, 0)).unwrap();

        self.repo
            .commit(
                None,
                &sig,
                &sig,
                "commit",
                &tree,
                &parents.iter().collect::<Vec<_>>(),
            )
            .unwrap()
    }

    fn set_head(&self, oid: Oid) {
        self.repo
            .reference("refs/heads/main", oid, true, "")
            .unwrap();
        self.repo.set_head("refs/heads/main").unwrap();
    }

    fn walker(&self) -> Walker {
        Walker::new(Path::new(&self.path), vec!["*.inc"]).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// (commit, rev-list count, paths)
fn collect(
    walker: &mut Walker,
    since: Option<Oid>,
    since_time: Option<i64>,
) -> Vec<(Oid, u64, Vec<String>)> {
    let since = since.map(|v| v.to_string());

    walker
        .walk(since.as_deref(), since_time)
        .unwrap()
        .map(|v| {
            (
                v[0].commit,
                v[0].count,
                v.iter()
                    .map(|b| b.path.to_string_lossy().into_owned())
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn walks_first_parent_history_with_rev_list_counts() {
    let f = Fixture::new("history");

    let root = f.commit(100, &[], &[("a.inc", "a1")]);
    let c2 = f.commit(200, &[root], &[("a.inc", "a2")]);
    // Side branch commit is newer than the mainline commit it gets merged over
    let side = f.commit(400, &[c2], &[("a.inc", "a2-side"), ("b.inc", "b1")]);
    let c3 = f.commit(300, &[c2], &[("a.inc", "a3")]);
    let merge = f.commit(500, &[c3, side], &[("a.inc", "a3-merged"), ("b.inc", "b1")]);
    let readme = f.commit(600, &[merge], &[("README", "readme")]);
    f.set_head(readme);

    let walked = collect(&mut f.walker(), None, None);

    assert_eq!(
        walked,
        vec![
            // Root commit is included
            (root, 1, vec!["a.inc".to_string()]),
            (c2, 2, vec!["a.inc".to_string()]),
            (c3, 3, vec!["a.inc".to_string()]),
            // Side branch content is only seen once merged, and counted in the merge
            (merge, 5, vec!["a.inc".to_string(), "b.inc".to_string()]),
        ]
    );

    let contents: Vec<Vec<u8>> = f
        .walker()
        .walk(None, None)
        .unwrap()
        .map(|v| v[0].content.clone())
        .collect();

    assert_eq!(walked.len(), contents.len());
    assert_eq!(contents[0], b"a1");
    assert_eq!(contents[3], b"a3-merged");
}

#[test]
fn walks_from_checkpoint() {
    let f = Fixture::new("checkpoint");

    let root = f.commit(100, &[], &[("a.inc", "a1")]);
    let c2 = f.commit(200, &[root], &[("a.inc", "a2")]);
    // Committer time earlier than the checkpoint, but landed after it
    let c3 = f.commit(150, &[c2], &[("a.inc", "a3")]);
    f.set_head(c3);

    assert_eq!(
        collect(&mut f.walker(), Some(c2), Some(200)),
        vec![(c3, 3, vec!["a.inc".to_string()])]
    );

    // Nothing new since HEAD
    assert!(collect(&mut f.walker(), Some(c3), Some(150)).is_empty());

    // Unknown checkpoint falls back to time
    let unknown = Oid::from_str("0123456789012345678901234567890123456789").unwrap();
    assert_eq!(
        collect(&mut f.walker(), Some(unknown), Some(200)),
        vec![(c2, 2, vec!["a.inc".to_string()])]
    );
}

#[test]
fn continues_through_reimported_history() {
    let f = Fixture::new("reimport");

    // Older history, only reachable through the second parent of a later merge
    let old1 = f.commit(100, &[], &[("a.inc", "a1")]);
    let old2 = f.commit(200, &[old1], &[("a.inc", "a2")]);

    // New root re-importing the same files
    let root = f.commit(300, &[], &[("a.inc", "a2"), ("README", "readme")]);
    let c2 = f.commit(400, &[root], &[("a.inc", "a3")]);
    let merge = f.commit(500, &[c2, old2], &[]);
    f.set_head(merge);

    assert_eq!(
        collect(&mut f.walker(), None, None),
        vec![
            (old1, 1, vec!["a.inc".to_string()]),
            (old2, 2, vec!["a.inc".to_string()]),
            // Unchanged from its predecessor, so the root itself isn't collected
            (c2, 2, vec!["a.inc".to_string()]),
        ]
    );
}

#[test]
fn unrelated_root_starts_from_empty_tree() {
    let f = Fixture::new("unrelated");

    let old = f.commit(100, &[], &[("a.inc", "a1")]);
    let root = f.commit(300, &[], &[("a.inc", "other")]);
    let merge = f.commit(500, &[root, old], &[]);
    f.set_head(merge);

    assert_eq!(
        collect(&mut f.walker(), None, None),
        vec![(root, 1, vec!["a.inc".to_string()])]
    );
}
