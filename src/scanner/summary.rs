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
