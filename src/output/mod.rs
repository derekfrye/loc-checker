use std::collections::BTreeMap;

use crate::scanner::{FileLocSummary, RootKind, ScannedFile, ScannerConfig};

pub fn print_report(config: &ScannerConfig, files: &[ScannedFile]) {
    match config.root_kind() {
        RootKind::File => print_file_root(config, files),
        RootKind::Directory => print_directory_root(config, files),
    }
}

fn print_file_root(config: &ScannerConfig, files: &[ScannedFile]) {
    if let Some(file) = files.first() {
        println!(
            ". {} ({})",
            config.root_label(),
            format_summary(&file.summary)
        );
    } else {
        println!(
            ". {} (no files matched language {})",
            config.root_label(),
            config.language.display_name()
        );
    }
}

fn print_directory_root(config: &ScannerConfig, files: &[ScannedFile]) {
    println!(". {}/", config.root_label());
    if files.is_empty() {
        println!(
            "|- no files matched language {}",
            config.language.display_name()
        );
        return;
    }

    let mut tree = TreeNode::default();
    for entry in files {
        tree.insert(
            &entry.relative_path.components().collect::<Vec<_>>(),
            &entry.summary,
        );
    }
    tree.sort();
    tree.print(0);
}

fn format_summary(summary: &FileLocSummary) -> String {
    let functions = if summary.top_function_locs.is_empty() {
        "none".to_string()
    } else {
        summary
            .top_function_locs
            .iter()
            .map(|loc| loc.to_string())
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
                    .or_insert_with(TreeNode::default)
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

    fn print(&self, depth: usize) {
        for (name, child) in &self.directories {
            println!("{}|- {}/", indent(depth), name);
            child.print(depth + 1);
        }

        for file in &self.files {
            println!(
                "{}|- {} ({})",
                indent(depth),
                file.name,
                format_summary(&file.summary)
            );
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

fn indent(depth: usize) -> String {
    let mut prefix = String::new();
    if depth > 0 {
        prefix.push_str(&"   ".repeat(depth));
    }
    prefix.push_str("|- ");
    prefix
}
