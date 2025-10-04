use std::path::PathBuf;

use clap::Parser;

use crate::language::Language;
use crate::output::OutputFormat;

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

    /// Output format for the rendered report
    #[arg(long = "output-format", value_enum, default_value_t = OutputFormat::Tree)]
    pub output_format: OutputFormat,
}

impl Cli {
    #[must_use]
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
