//! Snapshot tests over vendored upstream SourcePawn sources.
//!
//! Every file under `tests/corpus` is parsed and compared against
//! `tests/expected/<file>.json`. Run with `ALTERNATOR_BLESS=1` to write the
//! current output as the new expectation, then review the diff. Files that
//! fail to parse are recorded as `{ "error": ... }`.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("unable to read {}: {}", dir.display(), e))
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect(&path, out);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("inc") | Some("sp")
        ) {
            out.push(path);
        }
    }
}

/// First JSON path where two values differ
fn first_difference(expected: &Value, actual: &Value, path: String) -> Option<String> {
    match (expected, actual) {
        (Value::Object(a), Value::Object(b)) => {
            let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
            keys.sort();
            keys.dedup();
            keys.into_iter().find_map(|k| match (a.get(k), b.get(k)) {
                (Some(x), Some(y)) => first_difference(x, y, format!("{}/{}", path, k)),
                (Some(_), None) => Some(format!("{}/{} (missing)", path, k)),
                (None, _) => Some(format!("{}/{} (unexpected)", path, k)),
            })
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => a
            .iter()
            .zip(b)
            .enumerate()
            .find_map(|(i, (x, y))| first_difference(x, y, format!("{}/{}", path, i))),
        _ if expected == actual => None,
        _ => Some(format!("{}: expected {} got {}", path, expected, actual)),
    }
}

#[test]
fn corpus_matches_expected_output() {
    let corpus = root().join("corpus");
    let expected_dir = root().join("expected");
    let bless = std::env::var_os("ALTERNATOR_BLESS").is_some();

    let mut files = Vec::new();
    collect(&corpus, &mut files);
    assert!(files.len() > 100, "corpus is missing, run tests/update-corpus.sh");

    let mut failures = Vec::new();

    for file in &files {
        let rel = file.strip_prefix(&corpus).unwrap();
        let expected_path = expected_dir.join(format!("{}.json", rel.display()));

        let content = fs::read(file).unwrap();

        // Files that can't be parsed are snapshotted as their error
        let actual = match alternator::parse_bytes(&content) {
            // Round trip through Value for a stable key order
            Ok(strand) => serde_json::to_value(&strand).unwrap(),
            Err(e) => serde_json::json!({ "error": e.to_string() }),
        };
        let rendered = serde_json::to_string_pretty(&actual).unwrap() + "\n";

        if bless {
            fs::create_dir_all(expected_path.parent().unwrap()).unwrap();
            fs::write(&expected_path, rendered).unwrap();
            continue;
        }

        let expected: Value = match fs::read_to_string(&expected_path) {
            Ok(s) => serde_json::from_str(&s).unwrap(),
            Err(_) => {
                failures.push(format!("{}: no expected output", rel.display()));
                continue;
            }
        };

        if let Some(diff) = first_difference(&expected, &actual, String::new()) {
            failures.push(format!("{}: {}", rel.display(), diff));
        }
    }

    // Expectations without a corpus file are stale
    let mut expected_files = Vec::new();
    if expected_dir.exists() && !bless {
        collect_json(&expected_dir, &mut expected_files);
    }
    for e in expected_files {
        let rel = e.strip_prefix(&expected_dir).unwrap().to_string_lossy().to_string();
        let source = corpus.join(rel.trim_end_matches(".json"));
        if !source.exists() {
            failures.push(format!("{}: stale expectation", rel));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} corpus files differ (run with ALTERNATOR_BLESS=1 to update):\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
}

fn collect_json(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_json(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
            out.push(path);
        }
    }
}
