use std::path::Path;

use clap::ValueEnum;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, ValueEnum)]
pub enum Language {
    Auto,
    Rust,
    Csharp,
}

impl Language {
    #[must_use]
    pub fn matches(&self, path: &Path) -> bool {
        match self {
            Language::Auto => false,
            Language::Rust => path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("rs")),
            Language::Csharp => path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("cs")),
        }
    }

    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Auto => "auto",
            Language::Rust => "rust",
            Language::Csharp => "csharp",
        }
    }

    #[must_use]
    pub fn is_generated_path(&self, path: &Path) -> bool {
        match self {
            Language::Auto | Language::Rust => false,
            Language::Csharp => is_generated_csharp_path(path),
        }
    }
}

fn is_generated_csharp_path(path: &Path) -> bool {
    if path.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        value.eq_ignore_ascii_case("obj") || value.eq_ignore_ascii_case("bin")
    }) {
        return true;
    }

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let lower = file_name.to_ascii_lowercase();

    [
        ".designer.cs",
        ".generated.cs",
        ".g.cs",
        ".g.i.cs",
        ".assemblyinfo.cs",
        ".assemblyattributes.cs",
        ".razor.g.cs",
    ]
    .iter()
    .any(|suffix| lower.ends_with(suffix))
        || lower == "assemblyinfo.cs"
        || lower == "solutioninfo.cs"
        || lower.starts_with("temporarygeneratedfile_")
}
