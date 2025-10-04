use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    ImplItemFn, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemStatic, ItemStruct, ItemTrait,
    TraitItemFn,
};

use crate::scanner::summary::{ImplBlockLoc, ImplMethodLoc, NamedLoc, TraitMethodLoc};

use super::{ImplContext, ItemCollector};

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
