use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

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
