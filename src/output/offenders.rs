use crate::scanner::{FileLocSummary, ImplMethodLoc, NamedLoc, ScannedFile, TraitMethodLoc};

/// Thresholds used to filter out non-offending files and functions.
#[derive(Clone, Copy, Debug)]
pub struct OffenderFilter {
    pub max_loc_per_file: usize,
    pub max_loc_per_fn: usize,
}

impl OffenderFilter {
    #[must_use]
    pub const fn new(max_loc_per_file: usize, max_loc_per_fn: usize) -> Self {
        Self {
            max_loc_per_file,
            max_loc_per_fn,
        }
    }
}

/// Returns only the files (and function entries) that exceed the configured LOC limits.
#[must_use]
pub fn filter_files(files: &[ScannedFile], filter: &OffenderFilter) -> Vec<ScannedFile> {
    files
        .iter()
        .filter_map(|file| {
            let filtered_summary = filter_summary(&file.summary, filter);
            let file_exceeds = filtered_summary.total_loc > filter.max_loc_per_file;
            let has_function_offenders = summary_has_function_offenders(&filtered_summary);

            if file_exceeds || has_function_offenders {
                Some(ScannedFile {
                    relative_path: file.relative_path.clone(),
                    summary: filtered_summary,
                })
            } else {
                None
            }
        })
        .collect()
}

fn filter_summary(summary: &FileLocSummary, filter: &OffenderFilter) -> FileLocSummary {
    FileLocSummary {
        total_loc: summary.total_loc,
        top_functions: filter_named(&summary.top_functions, filter.max_loc_per_fn),
        file_scope_functions: filter_named(&summary.file_scope_functions, filter.max_loc_per_fn),
        impl_methods: filter_impl_methods(&summary.impl_methods, filter.max_loc_per_fn),
        trait_methods: filter_trait_methods(&summary.trait_methods, filter.max_loc_per_fn),
        test_functions: filter_named(&summary.test_functions, filter.max_loc_per_fn),
        struct_defs: summary.struct_defs.clone(),
        enum_defs: summary.enum_defs.clone(),
        trait_defs: summary.trait_defs.clone(),
        impl_blocks: summary.impl_blocks.clone(),
        consts: summary.consts.clone(),
        statics: summary.statics.clone(),
    }
}

fn filter_named(entries: &[NamedLoc], max_loc_per_fn: usize) -> Vec<NamedLoc> {
    entries
        .iter()
        .filter(|entry| entry.loc > max_loc_per_fn)
        .cloned()
        .collect()
}

fn filter_impl_methods(entries: &[ImplMethodLoc], max_loc_per_fn: usize) -> Vec<ImplMethodLoc> {
    entries
        .iter()
        .filter(|entry| entry.loc > max_loc_per_fn)
        .cloned()
        .collect()
}

fn filter_trait_methods(entries: &[TraitMethodLoc], max_loc_per_fn: usize) -> Vec<TraitMethodLoc> {
    entries
        .iter()
        .filter(|entry| entry.loc > max_loc_per_fn)
        .cloned()
        .collect()
}

fn summary_has_function_offenders(summary: &FileLocSummary) -> bool {
    !(summary.top_functions.is_empty()
        && summary.file_scope_functions.is_empty()
        && summary.impl_methods.is_empty()
        && summary.trait_methods.is_empty()
        && summary.test_functions.is_empty())
}
