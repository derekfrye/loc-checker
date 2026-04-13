use tree_sitter::Node;

use crate::scanner::summary::{ImplBlockLoc, ImplMethodLoc, NamedLoc, TraitMethodLoc};

use super::{ItemCollector, TypeContext};
use crate::scanner::analyze::csharp::nodes::{
    accessor_name, callable_name, first_identifier, has_modifier, name_for_node,
};

impl ItemCollector<'_> {
    pub(super) fn push_struct_like(&mut self, node: Node<'_>, source: &[u8]) {
        if let Some(entry) = self.named_entry(node, source) {
            self.struct_defs.push(entry);
        }
        self.push_type_context(node, source, false);
    }

    pub(super) fn push_interface(&mut self, node: Node<'_>, source: &[u8]) {
        if let Some(entry) = self.named_entry(node, source) {
            self.trait_defs.push(entry);
        }
        self.push_type_context(node, source, true);
    }

    pub(super) fn push_type_context(&mut self, node: Node<'_>, source: &[u8], is_interface: bool) {
        self.push_impl_block(node, source, is_interface);
        let name = name_for_node(node, source);
        self.type_stack.push(TypeContext { name, is_interface });
        self.visit_children(node, source);
        self.type_stack.pop();
    }

    pub(super) fn visit_children(&mut self, node: Node<'_>, source: &[u8]) {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                self.visit(cursor.node(), source);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    pub(super) fn push_impl_block(&mut self, node: Node<'_>, source: &[u8], is_interface: bool) {
        let Some(loc) = self.line_counter.record(node) else {
            return;
        };
        self.impl_blocks.push(ImplBlockLoc {
            target: name_for_node(node, source),
            trait_name: if is_interface {
                Some("interface".to_string())
            } else {
                None
            },
            loc,
        });
    }

    pub(super) fn named_entry(&self, node: Node<'_>, source: &[u8]) -> Option<NamedLoc> {
        let loc = self.line_counter.record(node)?;
        Some(NamedLoc {
            name: name_for_node(node, source),
            loc,
        })
    }

    pub(super) fn push_field(&mut self, node: Node<'_>, source: &[u8]) {
        let Some(loc) = self.line_counter.record(node) else {
            return;
        };
        let Some(name) = first_identifier(node, source) else {
            return;
        };
        let entry = NamedLoc { name, loc };
        if has_modifier(node, source, "const") {
            self.consts.push(entry);
        } else if has_modifier(node, source, "static") {
            self.statics.push(entry);
        }
        self.visit_children(node, source);
    }

    pub(super) fn push_callable(&mut self, node: Node<'_>, source: &[u8]) {
        if matches!(
            node.kind(),
            "property_declaration" | "indexer_declaration" | "event_declaration"
        ) && self.push_accessors(node, source)
        {
            self.visit_children(node, source);
            return;
        }

        let Some(loc) = self.line_counter.record(node) else {
            return;
        };
        let name = callable_name(node, source);
        self.function_summaries.push(NamedLoc {
            name: name.clone(),
            loc,
        });

        if let Some(context) = self.type_stack.last() {
            if context.is_interface {
                self.trait_methods.push(TraitMethodLoc {
                    trait_name: context.name.clone(),
                    method_name: name,
                    loc,
                });
            } else {
                self.impl_methods.push(ImplMethodLoc {
                    impl_target: context.name.clone(),
                    trait_name: None,
                    method_name: name,
                    loc,
                });
            }
        } else {
            self.file_scope_functions.push(NamedLoc { name, loc });
        }

        self.visit_children(node, source);
    }

    pub(super) fn push_event(&mut self, node: Node<'_>, source: &[u8]) {
        if let Some(entry) = self.named_entry(node, source) {
            self.event_defs.push(entry);
        }
        self.push_callable(node, source);
    }

    pub(super) fn push_event_field(&mut self, node: Node<'_>, source: &[u8]) {
        if let Some(entry) = self.named_entry(node, source) {
            self.event_defs.push(entry);
        }
        self.visit_children(node, source);
    }

    pub(super) fn push_local_function(&mut self, node: Node<'_>, source: &[u8]) {
        let Some(loc) = self.line_counter.record(node) else {
            return;
        };
        let name = format!("local {}", name_for_node(node, source));
        self.function_summaries.push(NamedLoc {
            name: name.clone(),
            loc,
        });
        self.file_scope_functions.push(NamedLoc { name, loc });
        self.visit_children(node, source);
    }

    pub(super) fn push_top_level_statement(&mut self, node: Node<'_>) {
        if let Some(loc) = self.line_counter.record(node) {
            self.top_level_statement_loc += loc;
        }
    }

    fn push_accessors(&mut self, node: Node<'_>, source: &[u8]) -> bool {
        let Some(accessors) = find_accessors(node) else {
            return false;
        };

        let parent_name = callable_name(node, source);
        let mut recorded_any = false;
        let mut cursor = accessors.walk();
        for accessor in accessors.children(&mut cursor) {
            if accessor.kind() != "accessor_declaration" {
                continue;
            }

            let Some(loc) = self.line_counter.record(accessor) else {
                continue;
            };
            recorded_any = true;
            self.record_callable(accessor_name(accessor, source, &parent_name), loc);
        }

        recorded_any
    }

    fn record_callable(&mut self, name: String, loc: usize) {
        self.function_summaries.push(NamedLoc {
            name: name.clone(),
            loc,
        });

        if let Some(context) = self.type_stack.last() {
            if context.is_interface {
                self.trait_methods.push(TraitMethodLoc {
                    trait_name: context.name.clone(),
                    method_name: name,
                    loc,
                });
            } else {
                self.impl_methods.push(ImplMethodLoc {
                    impl_target: context.name.clone(),
                    trait_name: None,
                    method_name: name,
                    loc,
                });
            }
        } else {
            self.file_scope_functions.push(NamedLoc { name, loc });
        }
    }
}

fn find_accessors(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.kind() == "accessor_list")
}
