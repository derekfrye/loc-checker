pub mod cli;
pub mod language;
pub mod output;
pub mod scanner;

use anyhow::Result;

pub fn run() -> Result<()> {
    let args = cli::Cli::parse();
    let config = scanner::ScannerConfig::try_from(args)?;
    let files = scanner::scan(&config)?;
    output::print_report(&config, &files);
    Ok(())
}
