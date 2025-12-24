use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;
use regex::Regex;

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

    if should_skip(&relative, config) {
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

        if should_skip(&relative, config) {
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

fn should_skip(relative: &Path, config: &ScannerConfig) -> bool {
    if config.excludes.iter().any(|ex| relative.starts_with(ex)) {
        return true;
    }

    let match_target = path_for_matching(relative);

    if matches_regexes(&match_target, &config.exclude_path_regexes) {
        return true;
    }

    if config.include_path_regexes.is_empty() {
        return false;
    }

    !matches_regexes(&match_target, &config.include_path_regexes)
}

fn matches_regexes(target: &str, regexes: &[Regex]) -> bool {
    regexes.iter().any(|re| re.is_match(target))
}

fn path_for_matching(relative: &Path) -> String {
    let raw = relative.to_string_lossy();
    if std::path::MAIN_SEPARATOR == '/' {
        raw.into_owned()
    } else {
        raw.replace(std::path::MAIN_SEPARATOR, "/")
    }
}
