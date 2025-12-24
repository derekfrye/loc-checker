# loc-checker

`loc-checker` is a command-line tool that walks a source tree, filters files by language, and reports total lines of code plus the LOC of the three longest functions per file in a tree-style view.

## Features

- CLI built with Clap 4 (`--path`, `--lang`, `--git-ignore-support`, `--exclude`, `--include-path`, `--exclude-path`)
- Language-aware file matching (Rust today, enum-based for future languages)
- Optional `.gitignore` honoring via `ignore` crate
- Outputs a `tree`-like summary with LOC metrics per file
- MVU-style architecture separates scanning logic from presentation

## Installation

```
cargo install --path .
```

## Usage

```
cargo run -- --path <ROOT> [--lang rust] [--git-ignore-support] [--exclude dir1,dir2] [--include-path <regex>] [--exclude-path <regex>]
```

Example:

```
cargo run -- --path ../uncommitted --git-ignore-support --exclude target --exclude-path ".*\\.gen\\.rs$"
```

## Development

```
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

## License

Released under the MIT License. See [LICENSE](LICENSE).

## Third-Party Notices

Third-party licenses are collected in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
