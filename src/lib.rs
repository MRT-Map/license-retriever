#![warn(
    clippy::as_ptr_cast_mut,
    clippy::as_underscore,
    clippy::bool_to_int_with_if,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::checked_conversions,
    clippy::clear_with_drain,
    clippy::clone_on_ref_ptr,
    clippy::cloned_instead_of_copied,
    clippy::cognitive_complexity,
    clippy::collection_is_never_read,
    clippy::copy_iterator,
    clippy::create_dir,
    clippy::default_trait_access,
    clippy::deref_by_slicing,
    clippy::doc_link_with_quotes,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::enum_glob_use,
    clippy::equatable_if_let,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filetype_is_file,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::fn_to_numeric_cast_any,
    clippy::from_iter_instead_of_collect,
    clippy::future_not_send,
    clippy::get_unwrap,
    clippy::if_not_else,
    clippy::if_then_some_else_none,
    clippy::implicit_hasher,
    //clippy::impl_trait_in_params,
    clippy::imprecise_flops,
    clippy::inconsistent_struct_constructor,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::items_after_statements,
    clippy::iter_not_returning_iterator,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::iter_with_drain,
    clippy::large_digit_groups,
    clippy::large_futures,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::manual_assert,
    clippy::manual_clamp,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::many_single_char_names,
    clippy::map_err_ignore,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::mismatching_type_param_order,
    clippy::missing_assert_message,
    clippy::missing_const_for_fn,
    clippy::missing_enforced_import_renames,
    clippy::multiple_unsafe_ops_per_block,
    clippy::must_use_candidate,
    clippy::mut_mut,
    clippy::naive_bytecount,
    clippy::needless_bitwise_bool,
    clippy::needless_collect,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::negative_feature_names,
    clippy::non_ascii_literal,
    clippy::non_send_fields_in_send_ty,
    clippy::or_fun_call,
    clippy::range_minus_one,
    clippy::range_plus_one,
    clippy::rc_buffer,
    clippy::redundant_closure_for_method_calls,
    clippy::redundant_else,
    clippy::redundant_feature_names,
    clippy::redundant_pub_crate,
    clippy::ref_option_ref,
    clippy::ref_patterns,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::return_self_not_must_use,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::semicolon_inside_block,
    clippy::separated_literal_suffix,
    clippy::significant_drop_in_scrutinee,
    clippy::significant_drop_tightening,
    clippy::single_match_else,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::string_slice,
    clippy::struct_excessive_bools,
    clippy::suboptimal_flops,
    clippy::suspicious_operation_groupings,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::trailing_empty_array,
    clippy::trait_duplication_in_bounds,
    clippy::transmute_ptr_to_ptr,
    clippy::transmute_undefined_repr,
    clippy::trivial_regex,
    clippy::trivially_copy_pass_by_ref,
    clippy::try_err,
    clippy::type_repetition_in_bounds,
    clippy::unchecked_duration_subtraction,
    clippy::undocumented_unsafe_blocks,
    clippy::unicode_not_nfc,
    clippy::uninlined_format_args,
    clippy::unnecessary_box_returns,
    clippy::unnecessary_join,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_self_imports,
    clippy::unnecessary_struct_initialization,
    clippy::unneeded_field_pattern,
    clippy::unnested_or_patterns,
    clippy::unreadable_literal,
    clippy::unsafe_derive_deserialize,
    clippy::unused_async,
    clippy::unused_peekable,
    clippy::unused_rounding,
    clippy::unused_self,
    clippy::unwrap_in_result,
    clippy::use_self,
    clippy::useless_let_if_seq,
    clippy::verbose_bit_mask,
    clippy::verbose_file_reads
)]
#![deny(
    clippy::derive_partial_eq_without_eq,
    clippy::match_bool,
    clippy::mem_forget,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::nonstandard_macro_braces,
    clippy::path_buf_push_overwrite,
    clippy::rc_mutex,
    clippy::wildcard_dependencies
)]

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
                path.to_owned(),
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
            .replace("(", "")
            .replace(")", "")
            .replace("/", " ")
            .split(" ")
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
