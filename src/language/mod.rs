use std::path::Path;

use clap::ValueEnum;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, ValueEnum)]
pub enum Language {
    Rust,
}

impl Language {
    pub fn matches(&self, path: &Path) -> bool {
        match self {
            Language::Rust => path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("rs"))
                .unwrap_or(false),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
        }
    }
}
