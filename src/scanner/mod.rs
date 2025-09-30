use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ImplItemFn, ItemFn, TraitItemFn};

use crate::cli::Cli;
use crate::language::Language;

#[derive(Clone, Debug)]
pub struct ScannerConfig {
    pub language: Language,
    canonical_root: PathBuf,
    pub git_ignore: bool,
    pub excludes: Vec<PathBuf>,
    root_kind: RootKind,
    root_label: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootKind {
    File,
    Directory,
}

#[derive(Clone, Debug)]
pub struct FileLocSummary {
    pub total_loc: usize,
    pub top_function_locs: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct ScannedFile {
    pub relative_path: PathBuf,
    pub summary: FileLocSummary,
}

impl ScannerConfig {
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.canonical_root
    }

    #[must_use]
    pub fn root_kind(&self) -> RootKind {
        self.root_kind
    }

    #[must_use]
    pub fn root_label(&self) -> &str {
        &self.root_label
    }
}

impl TryFrom<Cli> for ScannerConfig {
    type Error = anyhow::Error;

    fn try_from(cli: Cli) -> Result<Self> {
        let abs_path = if cli.path.is_absolute() {
            cli.path.clone()
        } else {
            std::env::current_dir()
                .context("failed to resolve current working directory")?
                .join(&cli.path)
        };

        let canonical_root = abs_path
            .canonicalize()
            .with_context(|| format!("failed to resolve path {}", abs_path.display()))?;

        let metadata = fs::metadata(&canonical_root).with_context(|| {
            format!("unable to access metadata for {}", canonical_root.display())
        })?;

        let root_kind = if metadata.is_dir() {
            RootKind::Directory
        } else if metadata.is_file() {
            RootKind::File
        } else {
            bail!(
                "path {} is not a regular file or directory",
                canonical_root.display()
            );
        };

        let root_label = canonical_root
            .file_name()
            .and_then(|name| name.to_str())
            .map_or_else(
                || canonical_root.display().to_string(),
                std::string::ToString::to_string,
            );

        let excludes = cli
            .exclude
            .into_iter()
            .map(|entry| entry.trim().to_string())
            .filter(|entry| !entry.is_empty())
            .map(PathBuf::from)
            .collect();

        Ok(Self {
            language: cli.lang,
            canonical_root,
            git_ignore: cli.git_ignore_support,
            excludes,
            root_kind,
            root_label,
        })
    }
}

/// Collects language-matching files under the configured root.
///
/// # Errors
/// Returns an error when filesystem access or source analysis fails.
pub fn scan(config: &ScannerConfig) -> Result<Vec<ScannedFile>> {
    match config.root_kind {
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

fn analyze_file(language: Language, path: &Path) -> Result<FileLocSummary> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;

    let total_loc = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    let top_function_locs = match language {
        Language::Rust => top_rust_function_locs(&source)?,
    };

    Ok(FileLocSummary {
        total_loc,
        top_function_locs,
    })
}

fn top_rust_function_locs(source: &str) -> Result<Vec<usize>> {
    let syntax = syn::parse_file(source).context("unable to parse Rust source")?;
    let mut collector = FnCollector::new(source);
    collector.visit_file(&syntax);
    let mut fn_locs = collector.into_counts();
    fn_locs.sort_by(|a, b| b.cmp(a));
    fn_locs.truncate(3);
    Ok(fn_locs)
}

struct FnCollector<'a> {
    lines: Vec<&'a str>,
    counts: Vec<usize>,
}

impl<'a> FnCollector<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            lines: source.lines().collect(),
            counts: Vec::new(),
        }
    }

    fn push_span(&mut self, span: Span) {
        if self.lines.is_empty() {
            return;
        }

        let start = span.start().line;
        let end = span.end().line;

        if start == 0 || end == 0 {
            return;
        }

        let start_index = start.saturating_sub(1);
        let mut end_index = end.saturating_sub(1);
        if end_index >= self.lines.len() {
            end_index = self.lines.len() - 1;
        }

        if end_index < start_index {
            return;
        }

        let mut count = 0usize;
        for idx in start_index..=end_index {
            if let Some(text) = self.lines.get(idx)
                && !text.trim().is_empty()
            {
                count += 1;
            }
        }

        self.counts.push(count);
    }

    fn into_counts(self) -> Vec<usize> {
        self.counts
    }
}

impl<'ast> Visit<'ast> for FnCollector<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.push_span(node.span());
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.push_span(node.span());
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        self.push_span(node.span());
        syn::visit::visit_trait_item_fn(self, node);
    }
}
