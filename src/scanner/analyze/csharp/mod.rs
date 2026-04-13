mod collector;
mod loc;
mod nodes;

use anyhow::{Result, anyhow};
use tree_sitter::Parser;

use crate::scanner::summary::FileLocSummary;

use super::CollectorParts;
use collector::ItemCollector;

pub(super) fn summarize_csharp_file(source: &str, total_loc: usize) -> Result<FileLocSummary> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
        .map_err(|err| anyhow!("failed to load C# grammar: {err}"))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow!("unable to parse C# source"))?;

    let mut collector = ItemCollector::new(source);
    collector.visit(tree.root_node(), source.as_bytes());
    let parts = collector.finish();

    Ok(FileLocSummary {
        total_loc,
        top_functions: top_functions(&parts),
        file_scope_functions: parts.file_scope_functions,
        impl_methods: parts.impl_methods,
        trait_methods: parts.trait_methods,
        test_functions: parts.test_functions,
        struct_defs: parts.struct_defs,
        enum_defs: parts.enum_defs,
        trait_defs: parts.trait_defs,
        delegate_defs: parts.delegate_defs,
        event_defs: parts.event_defs,
        impl_blocks: parts.impl_blocks,
        consts: parts.consts,
        statics: parts.statics,
    })
}

fn top_functions(parts: &CollectorParts) -> Vec<crate::scanner::NamedLoc> {
    let mut top_candidates = parts.function_summaries.clone();
    top_candidates.sort_by(|a, b| b.loc.cmp(&a.loc).then_with(|| a.name.cmp(&b.name)));
    top_candidates.truncate(3);
    top_candidates
}
