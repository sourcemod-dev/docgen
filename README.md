# SP DocGen

A documentation generator for SourcePawn includes

## Overview

- `chumbucket` - Chums through manifest and include files
- `edge_worker` - CloudFlare edge server worker for modifying content title and description
- `libalternator` - SourcePawn include parser built on [tree-sitter-sourcepawn](https://github.com/nilshelmig/tree-sitter-sourcepawn)
- `libschema` Shared types between documentation components
- `libwalker` Git history traversal/walker for sources
- `schema` - Typescript library of mostly symbol classes/interfaces for UI

## Download

You could download the latest Linux & Windows chumbucket from the artifacts of each commit. Mac builds are currently not being automatically built.

## Parser

`libalternator` parses one include (or plugin) file at a time into a strand of
documented symbols. It runs a small preprocessor (conditional compilation,
`#define`s, `#endinput`) and then walks the tree-sitter syntax tree. Doc
comments are attached to declarations the same way the old SourcePawn
`docparse` tool did, and parsed with [spdcp](https://github.com/rumblefrog/sp-dcp).

Dump the strand of a file:

```sh
cargo run -p alternator --example dump -- path/to/file.inc
```

### Tests

```sh
cargo test -p alternator -p schema
```

`libalternator/tests/corpus` holds upstream SourceMod and third-party sources
(see its `SOURCES.md`), and `libalternator/tests/expected` their expected
output. After an intended output change, update the snapshots with
`ALTERNATOR_BLESS=1 cargo test -p alternator --test corpus` and review the
diff. To pull newer upstream sources, bump the commits in
`libalternator/tests/update-corpus.sh` and re-run it.
