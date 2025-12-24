use std::path::PathBuf;

use clap::Parser;

use crate::language::Language;
use crate::output::{OffenderFilter, OutputFormat};

#[derive(Parser, Debug)]
#[command(author, version, about = "Count LOC across source files", long_about = None)]
pub struct Cli {
    /// Language to scan (currently only supports rust)
    #[arg(long, value_enum, default_value_t = Language::Rust)]
    pub lang: Language,

    /// Starting path to scan
    #[arg(long)]
    pub path: PathBuf,

    /// Enable .gitignore awareness when walking files
    #[arg(long)]
    pub git_ignore_support: bool,

    /// Comma-separated list of relative paths to exclude from scanning
    #[arg(long, value_delimiter = ',')]
    pub exclude: Vec<String>,

    /// Regex patterns to include files by relative path
    #[arg(long = "include-path", value_delimiter = ',')]
    pub include_path: Vec<String>,

    /// Regex patterns to exclude files by relative path
    #[arg(long = "exclude-path", value_delimiter = ',')]
    pub exclude_path: Vec<String>,

    /// Output format for the rendered report
    #[arg(long = "output-format", value_enum, default_value_t = OutputFormat::Tree)]
    pub output_format: OutputFormat,

    /// Only print files/functions exceeding LOC limits
    #[arg(
        long = "offenders-only",
        requires_all = ["offending_max_loc_per_file", "offending_max_loc_per_fn"]
    )]
    pub offenders_only: bool,

    /// Maximum allowed LOC per file when offenders-only mode is enabled
    #[arg(long = "offending-max-loc-per-file", value_parser = parse_positive_usize, requires = "offenders_only")]
    pub offending_max_loc_per_file: Option<usize>,

    /// Maximum allowed LOC per function when offenders-only mode is enabled
    #[arg(long = "offending-max-loc-per-fn", value_parser = parse_positive_usize, requires = "offenders_only")]
    pub offending_max_loc_per_fn: Option<usize>,
}

impl Cli {
    #[must_use]
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    /// # Panics
    /// Panics if `--offenders-only` is provided without the corresponding LOC thresholds.
    #[must_use]
    pub fn offender_filter(&self) -> Option<OffenderFilter> {
        if self.offenders_only {
            Some(OffenderFilter::new(
                self.offending_max_loc_per_file
                    .expect("clap enforces offenders-only requirements"),
                self.offending_max_loc_per_fn
                    .expect("clap enforces offenders-only requirements"),
            ))
        } else {
            None
        }
    }
}

fn parse_positive_usize(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|err| format!("failed to parse '{value}' as usize: {err}"))?;
    if parsed == 0 {
        return Err("value must be greater than zero".to_string());
    }
    Ok(parsed)
}
