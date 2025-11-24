pub mod app;
pub mod cli;
pub mod language;
pub mod output;
pub mod scanner;

use anyhow::Result;

/// Parses CLI arguments, drives the MVU application, and prints the rendered report.
///
/// # Errors
/// Returns an error when argument conversion or scanning fails.
pub fn run() -> Result<()> {
    let args = cli::Cli::parse();
    let config = scanner::ScannerConfig::try_from(&args)?;
    let offender_filter = args.offender_filter();
    let rendered = app::run(config, args.output_format, offender_filter.as_ref())?;
    println!("{rendered}");
    Ok(())
}
