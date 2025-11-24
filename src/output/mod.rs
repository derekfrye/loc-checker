mod json;
mod offenders;
mod tree;

use std::borrow::Cow;

use clap::ValueEnum;

use crate::scanner::{ScannedFile, ScannerConfig};

pub use offenders::OffenderFilter;

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
    offender_filter: Option<&OffenderFilter>,
) -> String {
    let filtered_files: Cow<'_, [ScannedFile]> = if let Some(filter) = offender_filter {
        Cow::Owned(offenders::filter_files(files, filter))
    } else {
        Cow::Borrowed(files)
    };

    match format {
        OutputFormat::Tree => {
            tree::render(config, filtered_files.as_ref(), offender_filter.is_some())
        }
        OutputFormat::Json => json::render(config, filtered_files.as_ref()),
    }
}
