use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use cargo_metadata::{Metadata, MetadataCommand, Package};
use config::Config;
use git2::{build::RepoBuilder, FetchOptions};
use itertools::Itertools;
use log::{debug, info, warn};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub mod config;
pub mod error;

fn get_metadata(manifest_path: Option<impl Into<PathBuf>>) -> Result<Metadata> {
    info!("Retrieving metadata");
    let mut cmd = MetadataCommand::new();
    if let Some(manifest_path) = manifest_path {
        cmd.manifest_path(manifest_path);
    }
    let metadata = cmd.exec()?;
    Ok(metadata)
}

fn get_packages(metadata: &Metadata) -> Vec<&Package> {
    let Some(resolve) = &metadata.resolve else {
        info!("No resolve, getting all packages");
        return metadata.packages.iter().collect();
    };
    let Some(root) = &resolve.root else {
        info!("No resolve root, getting all packages");
        return metadata.packages.iter().collect();
    };
    let mut packages = Vec::new();
    let mut to_eval = HashSet::from([root]);
    while let Some(id) = to_eval.iter().next().copied() {
        debug!("Evaluating {id}");
        to_eval.remove(id);
        let Some(package) = metadata.packages.iter().find(|a| a.id == *id) else {
            continue;
        };
        packages.push(package);
        let Some(node) = resolve.nodes.iter().find(|a| a.id == *id) else {
            continue;
        };
        for dep in &node.deps {
            if !packages.iter().any(|a| a.id == dep.pkg) {
                to_eval.insert(&dep.pkg);
            }
        }
    }
    packages
}

fn extract_licenses_from_repo_folder(path: &Path) -> Result<Vec<String>> {
    let mut licenses = vec![];
    for entry in path.read_dir()? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if !name.contains("license")
            && !name.contains("licence")
            && !name.contains("copyright")
            && !name.contains("copying")
        {
            continue;
        }
        info!("Found {:?}", entry.path());
        if entry.file_type()?.is_dir() {
            for entry2 in entry.path().read_dir()? {
                let entry2 = entry2?;
                if !entry2.file_type()?.is_dir() {
                    licenses.push(std::fs::read_to_string(entry2.path())?);
                }
            }
        } else {
            licenses.push(std::fs::read_to_string(entry.path())?);
        }
    }
    Ok(licenses)
}

fn clone_repo(id: &str, repository: &str) -> Result<bool> {
    let repository = repository
        .strip_suffix('/')
        .unwrap_or(repository)
        .split("/tree/")
        .next()
        .unwrap();
    let path = PathBuf::from(format!("{}/repo/{id}", std::env::var("OUT_DIR")?,));

    if path.exists() {
        return Ok(true);
    }

    info!("Cloning {repository} to {path:?}");
    if let Err(e) = RepoBuilder::new()
        .fetch_options({
            let mut fo = FetchOptions::new();
            fo.depth(1);
            fo
        })
        .clone(repository, &path)
    {
        if e.message() == "unexpected http status code: 404" {
            warn!("Repo {repository} not found");
            Ok(false)
        } else {
            Err(e.into())
        }
    } else {
        Ok(true)
    }
}

fn get_licenses(package: &Package) -> Result<Vec<String>> {
    if let Some(license_file) = package.license_file() {
        info!(
            "Retrieving license file at {license_file:?} for {}",
            package.name
        );
        return Ok(vec![std::fs::read_to_string(&license_file)?]);
    };

    let path = package
        .manifest_path
        .parent()
        .unwrap_or(&package.manifest_path);
    if path.exists() {
        let licenses = extract_licenses_from_repo_folder(path.as_std_path())?;
        if !licenses.is_empty() {
            return Ok(licenses);
        }
    }

    if let Some(repository) = &package.repository {
        let can_eval = clone_repo(&package.id.repr, repository)?;
        if can_eval {
            let path = PathBuf::from(format!(
                "{}/repo/{}",
                std::env::var("OUT_DIR")?,
                package.id.repr
            ));
            let paths = [
                path.clone(),
                path.join(&package.name),
                path.join("crates").join(&package.name),
            ];
            for path in paths {
                if path.exists() {
                    let licenses = extract_licenses_from_repo_folder(&path)?;
                    if !licenses.is_empty() {
                        return Ok(licenses);
                    }
                }
            }
        }
    }

    if let Some(license) = &package.license {
        let path = PathBuf::from(format!("{}/repo/@spdx", std::env::var("OUT_DIR")?));
        println!("{path:?}");
        let mut licenses = vec![];
        for license in license
            .replace(" AND ", " ")
            .replace(" OR ", " ")
            .replace(" WITH ", " ")
            .replace(['(', ')'], "")
            .replace('/', " ")
            .split(' ')
        {
            let path2 = path.join("text").join(format!("{license}.txt"));
            if path2.exists() {
                info!("Found {path2:?}");
                licenses.push(std::fs::read_to_string(path2)?);
            }
        }
        if !licenses.is_empty() {
            return Ok(licenses);
        }
    }

    Ok(vec![])
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LicenseRetriever(Vec<(Package, Vec<String>)>);
impl LicenseRetriever {
    pub fn from_config(config: &Config) -> Result<Self> {
        let metadata = get_metadata(config.manifest_path.as_ref())?;
        let packages = get_packages(&metadata);

        info!("Cloning spdx license repo");
        clone_repo("@spdx", "https://github.com/spdx/license-list-data")?;

        let licenses = packages
            .into_par_iter()
            .map(|a| {
                if let Some(licenses) = config.overrides.get(&a.name) {
                    return Ok((a.to_owned(), licenses.to_owned()));
                }
                Ok((a.to_owned(), get_licenses(a)?))
            })
            .collect::<Result<Vec<_>>>()?;

        let no_license = licenses
            .iter()
            .filter(|(a, b)| b.is_empty() && !config.ignored_crates.contains(&a.name))
            .map(|(a, _)| &a.name)
            .join(", ");
        if !no_license.is_empty() {
            if config.error_for_no_license {
                return Err(Error::NoLicensesFound(no_license));
            }
            warn!("No licenses found for: {no_license}");
        }

        Ok(Self(licenses))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(rmp_serde::to_vec_named(&self.0)?)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self(rmp_serde::from_slice(bytes)?))
    }

    pub fn save_in_out_dir(&self, file_name: &str) -> Result<()> {
        std::fs::write(
            PathBuf::from(std::env::var("OUT_DIR")?).join(file_name),
            self.to_bytes()?,
        )?;
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &<Self as IntoIterator>::Item> {
        self.0.iter()
    }
}

impl IntoIterator for LicenseRetriever {
    type Item = (Package, Vec<String>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[macro_export]
macro_rules! license_retriever_data {
    ($file_name:literal) => {
        license_retriever::LicenseRetriever::from_bytes(include_bytes!(concat!(
            std::env::var("OUT_DIR")?,
            "/",
            $file_name
        )))
    };
}
