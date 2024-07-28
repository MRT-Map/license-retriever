use std::{collections::HashSet, path::PathBuf};

#[derive(Clone, Default)]
pub struct Config {
    pub ignored_crates: HashSet<String>,
    pub manifest_path: Option<PathBuf>,
    pub panic_if_no_license_found: bool,
}
