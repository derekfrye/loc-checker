use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use regex::Regex;

use crate::cli::Cli;
use crate::language::Language;

#[derive(Clone, Debug)]
pub struct ScannerConfig {
    pub language: Language,
    canonical_root: PathBuf,
    pub git_ignore: bool,
    pub excludes: Vec<PathBuf>,
    pub include_path_regexes: Vec<Regex>,
    pub exclude_path_regexes: Vec<Regex>,
    root_kind: RootKind,
    root_label: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootKind {
    File,
    Directory,
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

    fn from_cli(cli: &Cli) -> Result<Self> {
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
            .iter()
            .map(|entry| entry.trim().to_string())
            .filter(|entry| !entry.is_empty())
            .map(PathBuf::from)
            .collect();

        let include_path_regexes = compile_regexes(&cli.include_path, "include-path")?;
        let exclude_path_regexes = compile_regexes(&cli.exclude_path, "exclude-path")?;
        let language = detect_language(cli.lang, &canonical_root, root_kind)?;

        Ok(Self {
            language,
            canonical_root,
            git_ignore: cli.git_ignore_support,
            excludes,
            include_path_regexes,
            exclude_path_regexes,
            root_kind,
            root_label,
        })
    }
}

impl TryFrom<Cli> for ScannerConfig {
    type Error = anyhow::Error;

    fn try_from(cli: Cli) -> Result<Self> {
        Self::from_cli(&cli)
    }
}

impl TryFrom<&Cli> for ScannerConfig {
    type Error = anyhow::Error;

    fn try_from(cli: &Cli) -> Result<Self> {
        Self::from_cli(cli)
    }
}

fn detect_language(requested: Language, root: &Path, root_kind: RootKind) -> Result<Language> {
    if requested != Language::Auto {
        return Ok(requested);
    }

    match root_kind {
        RootKind::File => detect_language_for_file(root),
        RootKind::Directory => detect_language_for_directory(root),
    }
}

fn detect_language_for_file(path: &Path) -> Result<Language> {
    if Language::Rust.matches(path) {
        return Ok(Language::Rust);
    }
    if Language::Csharp.matches(path) {
        return Ok(Language::Csharp);
    }

    bail!(
        "unable to detect language for file {}; pass --lang explicitly",
        path.display()
    );
}

fn detect_language_for_directory(root: &Path) -> Result<Language> {
    let has_cargo_toml = root.join("Cargo.toml").is_file();
    let has_dotnet_marker = fs::read_dir(root)
        .with_context(|| format!("failed to read directory {}", root.display()))?
        .filter_map(std::result::Result::ok)
        .any(|entry| {
            let path = entry.path();
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("sln") || ext.eq_ignore_ascii_case("csproj")
                })
        });

    let mut rust_files = 0usize;
    let mut csharp_files = 0usize;
    let mut builder = WalkBuilder::new(root);
    builder.hidden(false);
    builder.git_ignore(false);
    builder.git_global(false);
    builder.git_exclude(false);

    for entry in builder.build().flatten() {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        if Language::Rust.matches(path) {
            rust_files += 1;
        } else if Language::Csharp.matches(path) {
            let relative = path.strip_prefix(root).unwrap_or(path);
            if !Language::Csharp.is_generated_path(relative) {
                csharp_files += 1;
            }
        }

        if rust_files >= 64 && csharp_files >= 64 {
            break;
        }
    }

    if has_cargo_toml && !has_dotnet_marker {
        return Ok(Language::Rust);
    }
    if has_dotnet_marker && !has_cargo_toml {
        return Ok(Language::Csharp);
    }
    if rust_files > 0 && csharp_files == 0 {
        return Ok(Language::Rust);
    }
    if csharp_files > 0 && rust_files == 0 {
        return Ok(Language::Csharp);
    }
    if rust_files > csharp_files {
        return Ok(Language::Rust);
    }
    if csharp_files > rust_files {
        return Ok(Language::Csharp);
    }

    bail!(
        "unable to auto-detect language for {}; pass --lang explicitly",
        root.display()
    )
}

fn compile_regexes(values: &[String], label: &str) -> Result<Vec<Regex>> {
    values
        .iter()
        .map(|entry| entry.trim())
        .filter(|entry| !entry.is_empty())
        .map(|pattern| {
            Regex::new(pattern)
                .with_context(|| format!("invalid {label} regex: {pattern}"))
        })
        .collect()
}
