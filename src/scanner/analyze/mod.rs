mod collector;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::language::Language;

use super::summary::{FileLocSummary, ImplBlockLoc, ImplMethodLoc, NamedLoc, TraitMethodLoc};
use collector::ItemCollector;
use syn::visit::Visit;

pub fn analyze_file(language: Language, path: &Path) -> Result<FileLocSummary> {
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
    let parts = collector.finish();

    let mut top_candidates = parts.function_summaries.clone();
    top_candidates.sort_by(|a, b| b.loc.cmp(&a.loc).then_with(|| a.name.cmp(&b.name)));
    top_candidates.truncate(3);

    Ok(FileLocSummary {
        total_loc,
        top_functions: top_candidates,
        file_scope_functions: parts.file_scope_functions,
        impl_methods: parts.impl_methods,
        trait_methods: parts.trait_methods,
        test_functions: parts.test_functions,
        struct_defs: parts.struct_defs,
        enum_defs: parts.enum_defs,
        trait_defs: parts.trait_defs,
        impl_blocks: parts.impl_blocks,
        consts: parts.consts,
        statics: parts.statics,
    })
}

pub(super) struct CollectorParts {
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
    pub function_summaries: Vec<NamedLoc>,
}
