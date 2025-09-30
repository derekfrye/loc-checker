use std::collections::BTreeMap;

use crate::scanner::{FileLocSummary, RootKind, ScannedFile, ScannerConfig};

#[must_use]
pub fn render_report(config: &ScannerConfig, files: &[ScannedFile]) -> String {
    let lines = match config.root_kind() {
        RootKind::File => render_file_root(config, files),
        RootKind::Directory => render_directory_root(config, files),
    };

    lines.join("\n")
}

fn render_file_root(config: &ScannerConfig, files: &[ScannedFile]) -> Vec<String> {
    if let Some(file) = files.first() {
        vec![format!(
            ". {} ({})",
            config.root_label(),
            format_summary(&file.summary)
        )]
    } else {
        vec![format!(
            ". {} (no files matched language {})",
            config.root_label(),
            config.language.display_name()
        )]
    }
}

fn render_directory_root(config: &ScannerConfig, files: &[ScannedFile]) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(". {}/", config.root_label()));

    if files.is_empty() {
        lines.push(format!(
            "└── no files matched language {}",
            config.language.display_name()
        ));
        return lines;
    }

    let mut tree = TreeNode::default();
    for entry in files {
        tree.insert(
            &entry.relative_path.components().collect::<Vec<_>>(),
            &entry.summary,
        );
    }
    tree.sort();
    tree.render("", &mut lines);

    lines
}

fn format_summary(summary: &FileLocSummary) -> String {
    let functions = if summary.top_function_locs.is_empty() {
        "none".to_string()
    } else {
        summary
            .top_function_locs
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };

    format!("{} loc; max fns loc: {}", summary.total_loc, functions)
}

#[derive(Default)]
struct TreeNode {
    directories: BTreeMap<String, TreeNode>,
    files: Vec<FileEntry>,
}

impl TreeNode {
    fn insert(&mut self, components: &[std::path::Component<'_>], summary: &FileLocSummary) {
        if let Some((first, rest)) = components.split_first() {
            let name = component_to_string(first);
            if rest.is_empty() {
                self.files.push(FileEntry {
                    name,
                    summary: summary.clone(),
                });
            } else {
                self.directories
                    .entry(name)
                    .or_default()
                    .insert(rest, summary);
            }
        }
    }

    fn sort(&mut self) {
        self.files.sort_by(|a, b| a.name.cmp(&b.name));
        for child in self.directories.values_mut() {
            child.sort();
        }
    }

    fn render(&self, prefix: &str, lines: &mut Vec<String>) {
        enum Entry<'a> {
            Dir(&'a str, &'a TreeNode),
            File(&'a FileEntry),
        }

        let mut entries: Vec<Entry<'_>> = Vec::new();
        for (name, child) in &self.directories {
            entries.push(Entry::Dir(name, child));
        }
        for file in &self.files {
            entries.push(Entry::File(file));
        }

        let total = entries.len();
        for (index, entry) in entries.into_iter().enumerate() {
            let is_last = index + 1 == total;
            let connector = if is_last { "└──" } else { "├──" };
            match entry {
                Entry::Dir(name, child) => {
                    lines.push(format!("{prefix}{connector} {name}/"));
                    let next_prefix = if is_last {
                        format!("{prefix}    ")
                    } else {
                        format!("{prefix}│   ")
                    };
                    child.render(&next_prefix, lines);
                }
                Entry::File(file) => {
                    lines.push(format!(
                        "{}{} {} ({})",
                        prefix,
                        connector,
                        file.name,
                        format_summary(&file.summary)
                    ));
                }
            }
        }
    }
}

#[derive(Clone)]
struct FileEntry {
    name: String,
    summary: FileLocSummary,
}

fn component_to_string(component: &std::path::Component<'_>) -> String {
    component.as_os_str().to_string_lossy().into_owned()
}
