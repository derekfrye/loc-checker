use std::path::Path;

use clap::ValueEnum;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, ValueEnum)]
pub enum Language {
    Rust,
}

impl Language {
    #[must_use]
    pub fn matches(&self, path: &Path) -> bool {
        match self {
            Language::Rust => path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("rs")),
        }
    }

    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
        }
    }
}
