use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use super::analyze::analyze_file;
use super::config::{RootKind, ScannerConfig};
use super::summary::FileLocSummary;

#[derive(Clone, Debug)]
pub struct ScannedFile {
    pub relative_path: PathBuf,
    pub summary: FileLocSummary,
}

/// Collects language-matching files under the configured root.
///
/// # Errors
/// Returns an error when filesystem access or source analysis fails.
pub fn scan(config: &ScannerConfig) -> Result<Vec<ScannedFile>> {
    match config.root_kind() {
        RootKind::File => scan_file_root(config),
        RootKind::Directory => scan_directory_root(config),
    }
}

fn scan_file_root(config: &ScannerConfig) -> Result<Vec<ScannedFile>> {
    let path = config.root();
    if !config.language.matches(path) {
        return Ok(Vec::new());
    }

    let summary = analyze_file(config.language, path)?;
    let relative = path
        .file_name()
        .map_or_else(|| PathBuf::from(path), PathBuf::from);

    if should_exclude(&relative, &config.excludes) {
        return Ok(Vec::new());
    }

    Ok(vec![ScannedFile {
        relative_path: relative,
        summary,
    }])
}

fn scan_directory_root(config: &ScannerConfig) -> Result<Vec<ScannedFile>> {
    let mut builder = WalkBuilder::new(config.root());
    builder.sort_by_file_name(std::cmp::Ord::cmp);
    builder.hidden(false);

    if !config.git_ignore {
        builder.git_ignore(false);
        builder.git_global(false);
        builder.git_exclude(false);
    }

    let mut results = Vec::new();

    for entry in builder.build() {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let relative = match path.strip_prefix(config.root()) {
            Ok(rel) => rel.to_path_buf(),
            Err(_) => continue,
        };

        if should_exclude(&relative, &config.excludes) {
            continue;
        }

        if !config.language.matches(path) {
            continue;
        }

        let summary = analyze_file(config.language, path)
            .with_context(|| format!("failed to analyze {}", path.display()))?;

        results.push(ScannedFile {
            relative_path: relative,
            summary,
        });
    }

    results.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    Ok(results)
}

fn should_exclude(relative: &Path, excludes: &[PathBuf]) -> bool {
    excludes.iter().any(|ex| relative.starts_with(ex))
}
