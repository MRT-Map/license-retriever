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
    borrow::Cow,
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use async_lock::Semaphore;
use cargo_metadata::{Metadata, MetadataCommand, Package};
use futures::{executor, future};
use itertools::Itertools;
use lazy_regex::{lazy_regex, Lazy, Regex};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use surf::{StatusCode, Url};
use thiserror::Error;
use tl::ParserOptions;

static DOCS_RS_REGEX: Lazy<Regex> = lazy_regex!(r"^https?://docs\.rs/crate/(.*?)/(.*?)/source");
static GITHUB_FILE_REGEX: Lazy<Regex> =
    lazy_regex!(r"^https?://github\.com/(.*?)/(.*?)/blob/(.*?)/(.*)");
static GITHUB_HOME_PAGE_REGEX: Lazy<Regex> = lazy_regex!(r"^https?://github\.com/(.*?)/(.*)$");
static SEMAPHORE: Lazy<Semaphore> = Lazy::new(|| {
    Semaphore::new(std::env::var("MAX_GET_REQS").map_or(5, |a| a.parse::<usize>().unwrap()))
});

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("http error: {0:?} at {1}")]
    Http(surf::Error, Url),
    #[error("cargo metadata error: {0:?}")]
    Metadata(#[from] cargo_metadata::Error),
    #[error("html parse error: {0:?}")]
    Html(#[from] tl::ParseError),
    #[error("environment variable error: {0:?}")]
    Env(#[from] std::env::VarError),
    #[error("encoding error: {0:?}")]
    Encode(#[from] rmp_serde::encode::Error),
    #[error("decoding error: {0:?}")]
    Decode(#[from] rmp_serde::decode::Error),
    #[error("i/o error: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("url parsing error")]
    Url(#[from] url::ParseError),
    #[error("`{0} has no file list")]
    NotCratesIoFileList(Url),
    #[error("`{0}` has no files, or is not a github repo home page")]
    NotGithubHomePage(Url),
    #[error("`{0}` specified in `license_copying_crates` not found in package list")]
    CopierCrateNotFound(String),
    #[error("`{0}` specified in `license_copying_crates` not found in package list")]
    CopiedCrateNotFound(String),
    #[error("unknown error")]
    Unknown,
}
pub trait ErrWithUrl<T> {
    fn err_with_url(self, err: &Url) -> Result<T>;
}
impl<T> ErrWithUrl<T> for surf::Result<T> {
    fn err_with_url(self, err: &Url) -> Result<T> {
        self.map_err(|e| Error::Http(e, err.to_owned()))
    }
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Default)]
pub struct Config {
    pub license_text_overrides: HashMap<String, Vec<String>>,
    pub license_url_overrides: HashMap<String, Vec<Url>>,
    pub license_copying_crates: HashMap<String, String>,
    pub ignored_crates: HashSet<String>,
    pub manifest_path: Option<PathBuf>,
    pub panic_if_no_license_found: bool,
}
impl Config {
    #[must_use]
    pub const fn panic_if_no_license_found(mut self) -> Self {
        self.panic_if_no_license_found = true;
        self
    }
    #[must_use]
    pub fn copy_license(mut self, copier: impl Into<String>, copied: impl Into<String>) -> Self {
        self.license_copying_crates
            .insert(copier.into(), copied.into());
        self
    }
    #[must_use]
    pub fn override_license_text(
        mut self,
        crate_name: impl Into<String>,
        licenses: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.license_text_overrides.insert(
            crate_name.into(),
            licenses.into_iter().map(std::convert::Into::into).collect(),
        );
        self
    }
    #[must_use]
    pub fn override_license_url(
        mut self,
        crate_name: impl Into<String>,
        urls: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        self.license_url_overrides.insert(
            crate_name.into(),
            urls.into_iter()
                .map(|a| Url::parse(a.as_ref()))
                .collect::<Result<Vec<_>, url::ParseError>>()
                .expect("Malformed URL"),
        );
        self
    }
    #[must_use]
    pub fn ignore(mut self, crate_name: impl Into<String>) -> Self {
        self.ignored_crates.insert(crate_name.into());
        self
    }
}

fn get_metadata(manifest_path: Option<PathBuf>) -> Result<Metadata> {
    info!("Retrieving metadata");
    let mut cmd = MetadataCommand::new();
    if let Some(manifest_path) = manifest_path {
        cmd.manifest_path(manifest_path);
    }
    let metadata = cmd.exec()?;
    Ok(metadata)
}

async fn get_license_text_from_url(url: Url) -> Result<Option<String>> {
    debug!("Retrieving license from {url}");
    let lock = SEMAPHORE.acquire().await;
    let response = surf::get(&url).await.err_with_url(&url)?;
    if response.status() == StatusCode::NotFound {
        warn!("{url} not found");
        return Ok(None);
    }
    let content = surf::get(&url)
        .await
        .err_with_url(&url)?
        .body_string()
        .await
        .err_with_url(&url)?;
    drop(lock);
    if DOCS_RS_REGEX.is_match(url.as_str()) {
        if let Ok(dom) = tl::parse(&content, ParserOptions::default()) {
            if let Some(n) = dom
                .query_selector("code")
                .and_then(|mut a| a.next())
                .and_then(|nh| nh.get(dom.parser()))
            {
                debug!("docs.rs source found for {url}");
                return Ok(Some(n.inner_text(dom.parser()).into_owned()));
            }
        }
    }
    Ok(Some(content))
}

async fn get_license_texts_from_crates_io_package<'a>(
    package: &Package,
    use_latest: bool,
) -> Result<Vec<String>> {
    let list_url = Url::parse(&format!(
        "https://docs.rs/crate/{}/{}/source/",
        package.name,
        if use_latest {
            "latest".into()
        } else {
            package.version.to_string()
        }
    ))?;
    let lock = SEMAPHORE.acquire().await;
    let mut response = surf::get(&list_url).await.err_with_url(&list_url)?;
    if response.status() == StatusCode::NotFound {
        info!("docs.rs does not have {} {}", package.name, package.version);
        return Ok(vec![]);
    }
    let content = response.body_string().await.err_with_url(&list_url)?;
    drop(lock);
    let dom = tl::parse(&content, ParserOptions::default())?;
    let n = dom
        .query_selector("a")
        .ok_or_else(|| Error::NotCratesIoFileList(list_url.to_owned()))?;
    let futures = n
        .filter_map(|a| a.get(dom.parser()))
        .filter_map(|a| a.as_tag())
        .filter_map(|a| a.attributes().get("href")?)
        .map(|a| a.as_utf8_str())
        .filter(|a| a.to_lowercase().contains("license") || a.to_lowercase().contains("licence"))
        .filter_map(|a| a.strip_prefix("./").map(std::borrow::ToOwned::to_owned))
        .filter_map(|a| list_url.join(&a).ok())
        .inspect(|a| {
            debug!(
                "Found license in {a} for {} {}",
                package.name, package.version
            );
        })
        .map(get_license_text_from_url)
        .collect::<Vec<_>>();
    let texts = future::try_join_all(futures)
        .await?
        .into_iter()
        .flatten()
        .collect();
    Ok(texts)
}

async fn get_license_texts_from_github_repo<'a>(url: &Url) -> Result<Vec<String>> {
    let lock = SEMAPHORE.acquire().await;
    let mut response = surf::get(url).await.err_with_url(url)?;
    if response.status() == StatusCode::NotFound {
        info!("{} returned 404", url);
        return Ok(vec![]);
    }
    let content = response.body_string().await.err_with_url(url)?;
    drop(lock);
    let dom = tl::parse(&content, ParserOptions::default())?;
    let n = dom
        .query_selector("a.js-navigation-open.Link--primary")
        .ok_or_else(|| Error::NotGithubHomePage(url.to_owned()))?;
    let futures = n
        .filter_map(|a| a.get(dom.parser()))
        .filter_map(|a| a.as_tag())
        .map(|a| a.inner_text(dom.parser()))
        .filter(|a| a.to_lowercase().contains("license") || a.to_lowercase().contains("licence"))
        .filter_map(|a| {
            Url::parse(
                &GITHUB_FILE_REGEX.replace(&a, r"https://raw.githubusercontent.com/$1/$2/$3/$4"),
            )
            .ok()
        })
        .inspect(|a| debug!("Found license in {a} for {}", url))
        .map(get_license_text_from_url)
        .collect::<Vec<_>>();
    let texts = future::try_join_all(futures)
        .await?
        .into_iter()
        .flatten()
        .collect();
    Ok(texts)
}

async fn get_license_texts_from_package<'a>(
    package: &Package,
    config: &'a Config,
) -> Result<Cow<'a, [String]>> {
    if let Some(license) = config.license_text_overrides.get(&package.name) {
        info!(
            "Retrieving license for {} {} from text overrides",
            package.name, package.version
        );
        return Ok(Cow::Borrowed(license));
    }
    if let Some(urls) = config.license_url_overrides.get(&package.name) {
        info!(
            "Retrieving license for {} {} from url overrides",
            package.name, package.version
        );
        let futures = urls.iter().map(|a| get_license_text_from_url(a.to_owned()));
        let licenses = future::try_join_all(futures)
            .await?
            .into_iter()
            .flatten()
            .collect();
        return Ok(Cow::Owned(licenses));
    }

    info!(
        "Attempting to retrieve license for {} {} from crates.io",
        package.name, package.version
    );
    let mut licenses = get_license_texts_from_crates_io_package(package, false).await?;

    if licenses.is_empty() {
        info!(
            "Attempting to retrieve license for latest version of {} from crates.io",
            package.name
        );
        licenses = get_license_texts_from_crates_io_package(package, true).await?;
    }

    if licenses.is_empty() {
        if let Some(repo) = &package.repository.as_ref().and_then(|a| Url::parse(a).ok()) {
            if GITHUB_HOME_PAGE_REGEX.is_match(repo.as_str()) {
                info!("Attempting to retrieve license from {repo}",);
                licenses = get_license_texts_from_github_repo(repo).await?;
            }
        }
    }

    Ok(Cow::Owned(licenses))
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LicenseRetriever<'a>(Vec<(Package, Cow<'a, [String]>)>);
impl<'a> LicenseRetriever<'a> {
    pub async fn async_from_config(config: &'a Config) -> Result<LicenseRetriever<'a>> {
        let meta = get_metadata(Option::<PathBuf>::None)?;
        let futures = meta
            .packages
            .iter()
            .map(|p| get_license_texts_from_package(p, config))
            .collect::<Vec<_>>();
        let mut licenses = future::try_join_all(futures)
            .await?
            .into_iter()
            .zip_eq(&meta.packages)
            .map(|(licenses, package)| (package.to_owned(), licenses))
            .collect::<Vec<_>>();

        for (copier, copied) in &config.license_copying_crates {
            let copied_licenses = licenses
                .iter()
                .filter(|(p, _)| p.name == **copied)
                .sorted_by_key(|(_, a)| a.len())
                .last()
                .map(|(_, a)| a.clone())
                .ok_or_else(|| Error::CopiedCrateNotFound(copied.to_owned()))?;
            let (_, copier_licenses) = licenses
                .iter_mut()
                .find(|(p, _)| p.name == **copier)
                .ok_or_else(|| Error::CopierCrateNotFound(copier.to_owned()))?;
            *copier_licenses = copied_licenses;
        }

        let unlicensed_packages = licenses
            .iter()
            .filter_map(|(package, licenses)| licenses.is_empty().then_some(package))
            .filter(|a| !config.ignored_crates.contains(&a.name))
            .map(|package| format!("{} {}", package.name, package.version))
            .join(", ");
        if !unlicensed_packages.is_empty() {
            let msg = format!("No licenses found for: {unlicensed_packages}");
            if config.panic_if_no_license_found {
                panic!("{msg}");
            } else {
                warn!("{msg}");
            }
        }

        Ok(LicenseRetriever(licenses))
    }
    pub fn from_config(config: &'a Config) -> Result<LicenseRetriever<'a>> {
        executor::block_on(Self::async_from_config(config))
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(rmp_serde::to_vec_named(&self.0)?)
    }
    pub async fn async_save_in_out_dir(&self, file_name: &str) -> Result<()> {
        async_fs::write(
            PathBuf::from(std::env::var("OUT_DIR")?).join(file_name),
            self.to_bytes()?,
        )
        .await?;
        Ok(())
    }
    pub fn save_in_out_dir(&self, file_name: &str) -> Result<()> {
        executor::block_on(self.async_save_in_out_dir(file_name))
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<LicenseRetriever<'static>> {
        Ok(LicenseRetriever(rmp_serde::from_slice(bytes)?))
    }
    pub fn iter(&self) -> impl Iterator<Item = &<LicenseRetriever<'a> as IntoIterator>::Item> {
        self.0.iter()
    }
}

impl<'a> IntoIterator for LicenseRetriever<'a> {
    type Item = (Package, Cow<'a, [String]>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[macro_export]
macro_rules! license_retriever_data {
    ($file_name:literal) => {
        license_retriever::LicenseRetriever::from_bytes(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/",
            $file_name
        )))
    };
}
