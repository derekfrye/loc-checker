use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    ImplItemFn, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemStatic, ItemStruct, ItemTrait,
    TraitItemFn,
};

use crate::cli::Cli;
use crate::language::Language;

#[derive(Clone, Debug)]
pub struct ScannerConfig {
    pub language: Language,
    canonical_root: PathBuf,
    pub git_ignore: bool,
    pub excludes: Vec<PathBuf>,
    root_kind: RootKind,
    root_label: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootKind {
    File,
    Directory,
}

#[derive(Clone, Debug)]
pub struct FileLocSummary {
    pub total_loc: usize,
    pub top_functions: Vec<NamedLoc>,
    pub file_scope_functions: Vec<NamedLoc>,
    pub impl_methods: Vec<ImplMethodLoc>,
    pub trait_methods: Vec<TraitMethodLoc>,
    pub test_functions: Vec<NamedLoc>,
    pub struct_defs: Vec<NamedLoc>,
    pub enum_defs: Vec<NamedLoc>,
    pub trait_defs: Vec<NamedLoc>,
    pub impl_blocks: Vec<ImplBlockLoc>,
    pub consts: Vec<NamedLoc>,
    pub statics: Vec<NamedLoc>,
}

#[derive(Clone, Debug)]
pub struct NamedLoc {
    pub name: String,
    pub loc: usize,
}

#[derive(Clone, Debug)]
pub struct ImplMethodLoc {
    pub impl_target: String,
    pub trait_name: Option<String>,
    pub method_name: String,
    pub loc: usize,
}

#[derive(Clone, Debug)]
pub struct TraitMethodLoc {
    pub trait_name: String,
    pub method_name: String,
    pub loc: usize,
}

#[derive(Clone, Debug)]
pub struct ImplBlockLoc {
    pub target: String,
    pub trait_name: Option<String>,
    pub loc: usize,
}

#[derive(Clone, Debug)]
pub struct ScannedFile {
    pub relative_path: PathBuf,
    pub summary: FileLocSummary,
}

impl ScannerConfig {
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.canonical_root
    }

    #[must_use]
    pub fn root_kind(&self) -> RootKind {
        self.root_kind
    }

    #[must_use]
    pub fn root_label(&self) -> &str {
        &self.root_label
    }

    fn from_cli(cli: &Cli) -> Result<Self> {
        let abs_path = if cli.path.is_absolute() {
            cli.path.clone()
        } else {
            std::env::current_dir()
                .context("failed to resolve current working directory")?
                .join(&cli.path)
        };

        let canonical_root = abs_path
            .canonicalize()
            .with_context(|| format!("failed to resolve path {}", abs_path.display()))?;

        let metadata = fs::metadata(&canonical_root).with_context(|| {
            format!("unable to access metadata for {}", canonical_root.display())
        })?;

        let root_kind = if metadata.is_dir() {
            RootKind::Directory
        } else if metadata.is_file() {
            RootKind::File
        } else {
            bail!(
                "path {} is not a regular file or directory",
                canonical_root.display()
            );
        };

        let root_label = canonical_root
            .file_name()
            .and_then(|name| name.to_str())
            .map_or_else(
                || canonical_root.display().to_string(),
                std::string::ToString::to_string,
            );

        let excludes = cli
            .exclude
            .iter()
            .map(|entry| entry.trim().to_string())
            .filter(|entry| !entry.is_empty())
            .map(PathBuf::from)
            .collect();

        Ok(Self {
            language: cli.lang,
            canonical_root,
            git_ignore: cli.git_ignore_support,
            excludes,
            root_kind,
            root_label,
        })
    }
}

impl TryFrom<Cli> for ScannerConfig {
    type Error = anyhow::Error;

    fn try_from(cli: Cli) -> Result<Self> {
        Self::from_cli(&cli)
    }
}

impl TryFrom<&Cli> for ScannerConfig {
    type Error = anyhow::Error;

    fn try_from(cli: &Cli) -> Result<Self> {
        Self::from_cli(cli)
    }
}

/// Collects language-matching files under the configured root.
///
/// # Errors
/// Returns an error when filesystem access or source analysis fails.
pub fn scan(config: &ScannerConfig) -> Result<Vec<ScannedFile>> {
    match config.root_kind {
        RootKind::File => scan_file_root(config),
        RootKind::Directory => scan_directory_root(config),
    }
}

fn scan_file_root(config: &ScannerConfig) -> Result<Vec<ScannedFile>> {
    let path = config.root();
    if !config.language.matches(path) {
        return Ok(Vec::new());
    }

    let summary = analyze_file(config.language, path)?;
    let relative = path
        .file_name()
        .map_or_else(|| PathBuf::from(path), PathBuf::from);

    if should_exclude(&relative, &config.excludes) {
        return Ok(Vec::new());
    }

    Ok(vec![ScannedFile {
        relative_path: relative,
        summary,
    }])
}

fn scan_directory_root(config: &ScannerConfig) -> Result<Vec<ScannedFile>> {
    let mut builder = WalkBuilder::new(config.root());
    builder.sort_by_file_name(std::cmp::Ord::cmp);
    builder.hidden(false);

    if !config.git_ignore {
        builder.git_ignore(false);
        builder.git_global(false);
        builder.git_exclude(false);
    }

    let mut results = Vec::new();

    for entry in builder.build() {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let relative = match path.strip_prefix(config.root()) {
            Ok(rel) => rel.to_path_buf(),
            Err(_) => continue,
        };

        if should_exclude(&relative, &config.excludes) {
            continue;
        }

        if !config.language.matches(path) {
            continue;
        }

        let summary = analyze_file(config.language, path)
            .with_context(|| format!("failed to analyze {}", path.display()))?;

        results.push(ScannedFile {
            relative_path: relative,
            summary,
        });
    }

    results.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    Ok(results)
}

fn should_exclude(relative: &Path, excludes: &[PathBuf]) -> bool {
    excludes.iter().any(|ex| relative.starts_with(ex))
}

fn analyze_file(language: Language, path: &Path) -> Result<FileLocSummary> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;

    let total_loc = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    match language {
        Language::Rust => summarize_rust_file(&source, total_loc),
    }
}

fn summarize_rust_file(source: &str, total_loc: usize) -> Result<FileLocSummary> {
    let syntax = syn::parse_file(source).context("unable to parse Rust source")?;
    let mut collector = ItemCollector::new(source);
    collector.visit_file(&syntax);

    let mut top_candidates = collector.function_summaries.clone();
    top_candidates.sort_by(|a, b| b.loc.cmp(&a.loc).then_with(|| a.name.cmp(&b.name)));
    top_candidates.truncate(3);

    Ok(FileLocSummary {
        total_loc,
        top_functions: top_candidates,
        file_scope_functions: collector.file_scope_functions,
        impl_methods: collector.impl_methods,
        trait_methods: collector.trait_methods,
        test_functions: collector.test_functions,
        struct_defs: collector.struct_defs,
        enum_defs: collector.enum_defs,
        trait_defs: collector.trait_defs,
        impl_blocks: collector.impl_blocks,
        consts: collector.consts,
        statics: collector.statics,
    })
}

struct ItemCollector<'a> {
    lines: Vec<&'a str>,
    file_scope_functions: Vec<NamedLoc>,
    impl_methods: Vec<ImplMethodLoc>,
    trait_methods: Vec<TraitMethodLoc>,
    test_functions: Vec<NamedLoc>,
    struct_defs: Vec<NamedLoc>,
    enum_defs: Vec<NamedLoc>,
    trait_defs: Vec<NamedLoc>,
    impl_blocks: Vec<ImplBlockLoc>,
    consts: Vec<NamedLoc>,
    statics: Vec<NamedLoc>,
    function_summaries: Vec<NamedLoc>,
    impl_stack: Vec<ImplContext>,
    trait_stack: Vec<String>,
}

#[derive(Clone, Debug)]
struct ImplContext {
    target: String,
    trait_name: Option<String>,
}

impl<'a> ItemCollector<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            lines: source.lines().collect(),
            file_scope_functions: Vec::new(),
            impl_methods: Vec::new(),
            trait_methods: Vec::new(),
            test_functions: Vec::new(),
            struct_defs: Vec::new(),
            enum_defs: Vec::new(),
            trait_defs: Vec::new(),
            impl_blocks: Vec::new(),
            consts: Vec::new(),
            statics: Vec::new(),
            function_summaries: Vec::new(),
            impl_stack: Vec::new(),
            trait_stack: Vec::new(),
        }
    }

    fn record_loc(&self, span: Span) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }

        let start = span.start().line;
        let end = span.end().line;

        if start == 0 || end == 0 {
            return None;
        }

        let start_index = start.saturating_sub(1);
        let mut end_index = end.saturating_sub(1);
        if end_index >= self.lines.len() {
            end_index = self.lines.len().saturating_sub(1);
        }

        if end_index < start_index {
            return None;
        }

        let count = self.lines[start_index..=end_index]
            .iter()
            .filter(|line| !line.trim().is_empty())
            .count();

        if count == 0 { None } else { Some(count) }
    }

    fn push_function_summary(&mut self, name: &str, span: Span) -> Option<usize> {
        let loc = self.record_loc(span)?;
        self.function_summaries.push(NamedLoc {
            name: name.to_string(),
            loc,
        });
        Some(loc)
    }
}

impl<'ast> Visit<'ast> for ItemCollector<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let fn_name = node.sig.ident.to_string();
        if let Some(loc) = self.push_function_summary(&fn_name, node.span()) {
            let is_test = node.attrs.iter().any(|attr| attr.path().is_ident("test"));
            let entry = NamedLoc { name: fn_name, loc };
            if is_test {
                self.test_functions.push(entry);
            } else {
                self.file_scope_functions.push(entry);
            }
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if let Some(loc) = self.record_loc(node.span()) {
            self.struct_defs.push(NamedLoc {
                name: node.ident.to_string(),
                loc,
            });
        }
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        if let Some(loc) = self.record_loc(node.span()) {
            self.enum_defs.push(NamedLoc {
                name: node.ident.to_string(),
                loc,
            });
        }
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_const(&mut self, node: &'ast ItemConst) {
        if let Some(loc) = self.record_loc(node.span()) {
            self.consts.push(NamedLoc {
                name: node.ident.to_string(),
                loc,
            });
        }
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_static(&mut self, node: &'ast ItemStatic) {
        if let Some(loc) = self.record_loc(node.span()) {
            self.statics.push(NamedLoc {
                name: node.ident.to_string(),
                loc,
            });
        }
        syn::visit::visit_item_static(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        if let Some(loc) = self.record_loc(node.span()) {
            self.trait_defs.push(NamedLoc {
                name: node.ident.to_string(),
                loc,
            });
        }
        self.trait_stack.push(node.ident.to_string());
        syn::visit::visit_item_trait(self, node);
        self.trait_stack.pop();
    }

    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        if node.default.is_some()
            && let Some(trait_name) = self.trait_stack.last().cloned()
        {
            let display = format!("trait {}::{}", trait_name, node.sig.ident);
            if let Some(loc) = self.push_function_summary(&display, node.span()) {
                self.trait_methods.push(TraitMethodLoc {
                    trait_name,
                    method_name: node.sig.ident.to_string(),
                    loc,
                });
            }
        }
        syn::visit::visit_trait_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        let target = node.self_ty.to_token_stream().to_string();
        let trait_name = node
            .trait_
            .as_ref()
            .map(|(_, path, _)| path.to_token_stream().to_string());

        if let Some(loc) = self.record_loc(node.span()) {
            self.impl_blocks.push(ImplBlockLoc {
                target: target.clone(),
                trait_name: trait_name.clone(),
                loc,
            });
        }

        self.impl_stack.push(ImplContext { target, trait_name });

        syn::visit::visit_item_impl(self, node);
        self.impl_stack.pop();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        if let Some(context) = self.impl_stack.last().cloned() {
            let display = match &context.trait_name {
                Some(trait_name) => {
                    format!("{} for {}::{}", trait_name, context.target, node.sig.ident)
                }
                None => format!("{}::{}", context.target, node.sig.ident),
            };
            if let Some(loc) = self.push_function_summary(&display, node.span()) {
                self.impl_methods.push(ImplMethodLoc {
                    impl_target: context.target,
                    trait_name: context.trait_name,
                    method_name: node.sig.ident.to_string(),
                    loc,
                });
            }
        }
        syn::visit::visit_impl_item_fn(self, node);
    }
}
