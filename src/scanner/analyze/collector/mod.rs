mod visit;

use proc_macro2::Span;

use crate::scanner::summary::{ImplBlockLoc, ImplMethodLoc, NamedLoc, TraitMethodLoc};

use super::CollectorParts;

pub(super) struct ItemCollector<'a> {
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
    pub(super) fn new(source: &'a str) -> Self {
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

    pub(super) fn finish(self) -> CollectorParts {
        CollectorParts {
            file_scope_functions: self.file_scope_functions,
            impl_methods: self.impl_methods,
            trait_methods: self.trait_methods,
            test_functions: self.test_functions,
            struct_defs: self.struct_defs,
            enum_defs: self.enum_defs,
            trait_defs: self.trait_defs,
            impl_blocks: self.impl_blocks,
            consts: self.consts,
            statics: self.statics,
            function_summaries: self.function_summaries,
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
