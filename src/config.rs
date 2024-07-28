use std::{collections::HashSet, path::PathBuf};

#[derive(Clone, Default)]
pub struct Config {
    pub ignored_crates: HashSet<String>,
    pub manifest_path: Option<PathBuf>,
    pub error_for_no_license: bool,
}
