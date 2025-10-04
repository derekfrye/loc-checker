use serde_json::{Value, json};

use crate::scanner::{
    ImplBlockLoc, ImplMethodLoc, NamedLoc, RootKind, ScannedFile, ScannerConfig, TraitMethodLoc,
};

#[must_use]
pub fn render(config: &ScannerConfig, files: &[ScannedFile]) -> String {
    let total_loc: usize = files.iter().map(|file| file.summary.total_loc).sum();
    let excludes = config
        .excludes
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    let files_json = files
        .iter()
        .map(|file| {
            let summary = &file.summary;
            json!({
                "path": file.relative_path.to_string_lossy(),
                "summary": {
                    "total_loc": summary.total_loc,
                    "top_functions": named_locs_to_json(&summary.top_functions),
                    "file_scope_functions": named_locs_to_json(&summary.file_scope_functions),
                    "impl_methods": impl_methods_to_json(&summary.impl_methods),
                    "trait_methods": trait_methods_to_json(&summary.trait_methods),
                    "test_functions": named_locs_to_json(&summary.test_functions),
                    "structs": named_locs_to_json(&summary.struct_defs),
                    "enums": named_locs_to_json(&summary.enum_defs),
                    "traits": named_locs_to_json(&summary.trait_defs),
                    "impl_blocks": impl_blocks_to_json(&summary.impl_blocks),
                    "consts": named_locs_to_json(&summary.consts),
                    "statics": named_locs_to_json(&summary.statics),
                }
            })
        })
        .collect::<Vec<_>>();

    let report = json!({
        "root": {
            "label": config.root_label(),
            "kind": match config.root_kind() {
                RootKind::File => "file",
                RootKind::Directory => "directory",
            },
            "path": config.root().display().to_string(),
        },
        "language": config.language.display_name(),
        "git_ignore": config.git_ignore,
        "excludes": excludes,
        "totals": {
            "files": files.len(),
            "total_loc": total_loc,
        },
        "files": files_json,
    });

    serde_json::to_string_pretty(&report).expect("json serialization should succeed")
}

fn named_locs_to_json(entries: &[NamedLoc]) -> Vec<Value> {
    let mut items = entries.to_vec();
    items.sort_by(|a, b| b.loc.cmp(&a.loc).then_with(|| a.name.cmp(&b.name)));
    items
        .into_iter()
        .map(|entry| {
            json!({
                "name": entry.name,
                "loc": entry.loc,
            })
        })
        .collect()
}

fn impl_methods_to_json(entries: &[ImplMethodLoc]) -> Vec<Value> {
    let mut items = entries.to_vec();
    items.sort_by(|a, b| {
        b.loc.cmp(&a.loc).then_with(|| {
            let trait_cmp = a.trait_name.cmp(&b.trait_name);
            if trait_cmp == std::cmp::Ordering::Equal {
                a.impl_target
                    .cmp(&b.impl_target)
                    .then(a.method_name.cmp(&b.method_name))
            } else {
                trait_cmp
            }
        })
    });

    items
        .into_iter()
        .map(|entry| {
            json!({
                "impl_target": entry.impl_target,
                "trait_name": entry.trait_name,
                "method_name": entry.method_name,
                "loc": entry.loc,
            })
        })
        .collect()
}

fn trait_methods_to_json(entries: &[TraitMethodLoc]) -> Vec<Value> {
    let mut items = entries.to_vec();
    items.sort_by(|a, b| {
        b.loc.cmp(&a.loc).then_with(|| {
            let trait_cmp = a.trait_name.cmp(&b.trait_name);
            if trait_cmp == std::cmp::Ordering::Equal {
                a.method_name.cmp(&b.method_name)
            } else {
                trait_cmp
            }
        })
    });

    items
        .into_iter()
        .map(|entry| {
            json!({
                "trait_name": entry.trait_name,
                "method_name": entry.method_name,
                "loc": entry.loc,
            })
        })
        .collect()
}

fn impl_blocks_to_json(entries: &[ImplBlockLoc]) -> Vec<Value> {
    let mut items = entries.to_vec();
    items.sort_by(|a, b| {
        b.loc.cmp(&a.loc).then_with(|| {
            let trait_cmp = a.trait_name.cmp(&b.trait_name);
            if trait_cmp == std::cmp::Ordering::Equal {
                a.target.cmp(&b.target)
            } else {
                trait_cmp
            }
        })
    });

    items
        .into_iter()
        .map(|entry| {
            json!({
                "impl_target": entry.target,
                "trait_name": entry.trait_name,
                "loc": entry.loc,
            })
        })
        .collect()
}
