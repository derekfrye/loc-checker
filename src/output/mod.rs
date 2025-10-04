mod json;
mod tree;

use clap::ValueEnum;

use crate::scanner::{ScannedFile, ScannerConfig};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Tree,
    Json,
}

#[must_use]
pub fn render_report(
    config: &ScannerConfig,
    files: &[ScannedFile],
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Tree => tree::render(config, files),
        OutputFormat::Json => json::render(config, files),
    }
}
