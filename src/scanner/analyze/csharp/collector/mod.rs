mod members;

use tree_sitter::Node;

use crate::scanner::summary::{ImplBlockLoc, ImplMethodLoc, NamedLoc, TraitMethodLoc};

use super::super::CollectorParts;
use super::loc::LineCounter;

pub(super) struct ItemCollector<'a> {
    line_counter: LineCounter<'a>,
    file_scope_functions: Vec<NamedLoc>,
    impl_methods: Vec<ImplMethodLoc>,
    trait_methods: Vec<TraitMethodLoc>,
    test_functions: Vec<NamedLoc>,
    struct_defs: Vec<NamedLoc>,
    enum_defs: Vec<NamedLoc>,
    trait_defs: Vec<NamedLoc>,
    delegate_defs: Vec<NamedLoc>,
    event_defs: Vec<NamedLoc>,
    impl_blocks: Vec<ImplBlockLoc>,
    consts: Vec<NamedLoc>,
    statics: Vec<NamedLoc>,
    function_summaries: Vec<NamedLoc>,
    top_level_statement_loc: usize,
    type_stack: Vec<TypeContext>,
}

#[derive(Clone)]
struct TypeContext {
    name: String,
    is_interface: bool,
}

impl<'a> ItemCollector<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self {
            line_counter: LineCounter::new(source),
            file_scope_functions: Vec::new(),
            impl_methods: Vec::new(),
            trait_methods: Vec::new(),
            test_functions: Vec::new(),
            struct_defs: Vec::new(),
            enum_defs: Vec::new(),
            trait_defs: Vec::new(),
            delegate_defs: Vec::new(),
            event_defs: Vec::new(),
            impl_blocks: Vec::new(),
            consts: Vec::new(),
            statics: Vec::new(),
            function_summaries: Vec::new(),
            top_level_statement_loc: 0,
            type_stack: Vec::new(),
        }
    }

    pub(super) fn finish(mut self) -> CollectorParts {
        if self.top_level_statement_loc > 0 {
            let entry = NamedLoc {
                name: "top-level statements".to_string(),
                loc: self.top_level_statement_loc,
            };
            self.function_summaries.push(entry.clone());
            self.file_scope_functions.push(entry);
        }

        CollectorParts {
            file_scope_functions: self.file_scope_functions,
            impl_methods: self.impl_methods,
            trait_methods: self.trait_methods,
            test_functions: self.test_functions,
            struct_defs: self.struct_defs,
            enum_defs: self.enum_defs,
            trait_defs: self.trait_defs,
            delegate_defs: self.delegate_defs,
            event_defs: self.event_defs,
            impl_blocks: self.impl_blocks,
            consts: self.consts,
            statics: self.statics,
            function_summaries: self.function_summaries,
        }
    }

    pub(super) fn visit(&mut self, node: Node<'_>, source: &[u8]) {
        let handled = match node.kind() {
            "class_declaration" | "struct_declaration" | "record_declaration" => {
                self.push_struct_like(node, source);
                true
            }
            "interface_declaration" => {
                self.push_interface(node, source);
                true
            }
            "enum_declaration" => {
                if let Some(entry) = self.named_entry(node, source) {
                    self.enum_defs.push(entry);
                }
                false
            }
            "delegate_declaration" => {
                if let Some(entry) = self.named_entry(node, source) {
                    self.delegate_defs.push(entry);
                }
                false
            }
            "event_declaration" => {
                self.push_event(node, source);
                true
            }
            "event_field_declaration" => {
                self.push_event_field(node, source);
                true
            }
            "field_declaration" => {
                self.push_field(node, source);
                true
            }
            "global_statement" => {
                self.push_top_level_statement(node);
                true
            }
            "method_declaration"
            | "constructor_declaration"
            | "destructor_declaration"
            | "operator_declaration"
            | "conversion_operator_declaration"
            | "property_declaration"
            | "indexer_declaration" => {
                self.push_callable(node, source);
                true
            }
            "local_function_statement" => {
                self.push_local_function(node, source);
                true
            }
            _ => false,
        };

        if !handled {
            self.visit_children(node, source);
        }
    }
}
