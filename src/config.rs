use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Clone, Default)]
pub struct Config {
    pub overrides: HashMap<String, Vec<String>>,
    pub ignored_crates: HashSet<String>,
    pub manifest_path: Option<PathBuf>,
    pub error_for_no_license: bool,
}
