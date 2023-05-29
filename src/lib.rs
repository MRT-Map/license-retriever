use std::{borrow::Cow, collections::HashMap, path::PathBuf};

use cargo_metadata::{Metadata, MetadataCommand, Package};
use futures::{executor, future};
use itertools::Itertools;
use lazy_regex::{lazy_regex, Lazy, Regex};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use surf::{StatusCode, Url};
use thiserror::Error;
use tl::ParserOptions;

static DOCS_RS_REGEX: Lazy<Regex> = lazy_regex!(r"^https?://docs\.rs/crate/.*?/.*?/source");

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("http error: {0:?}")]
    Http(surf::Error),
    #[error("cargo metadata error: {0:?}")]
    Metadata(#[from] cargo_metadata::Error),
    #[error("html parse error: {0:?}")]
    Html(#[from] tl::ParseError),
    #[error("Environment variable error: {0:?}")]
    Env(#[from] std::env::VarError),
    #[error("Encoding error: {0:?}")]
    Encode(#[from] rmp_serde::encode::Error),
    #[error("Decoding error: {0:?}")]
    Decode(#[from] rmp_serde::decode::Error),
    #[error("I/O error: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("No package menu in `{0}`")]
    NoPackageMenu(String),
    #[error("unknown error")]
    Unknown,
}
impl From<surf::Error> for Error {
    fn from(err: surf::Error) -> Self {
        Error::Http(err)
    }
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Default)]
pub struct Config {
    pub license_text_overrides: HashMap<String, Vec<String>>,
    pub license_url_overrides: HashMap<String, Vec<Url>>,
    pub manifest_path: Option<PathBuf>,
}
impl Config {
    pub fn manifest_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.manifest_path = Some(path.into());
        self
    }
    pub fn override_license_text(
        mut self,
        crate_name: impl Into<String>,
        licenses: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.license_text_overrides.insert(
            crate_name.into(),
            licenses.into_iter().map(|a| a.into()).collect(),
        );
        self
    }
    pub fn override_license_url(
        mut self,
        crate_name: impl Into<String>,
        urls: impl IntoIterator<Item = impl Into<Url>>,
    ) -> Self {
        self.license_url_overrides.insert(
            crate_name.into(),
            urls.into_iter().map(|a| a.into()).collect(),
        );
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

async fn get_license_text_from_url(url: Url) -> Result<String> {
    debug!("Retrieving license from {url}");
    let content = surf::get(&url).await?.body_string().await?;
    if DOCS_RS_REGEX.is_match(url.as_str()) {
        if let Ok(dom) = tl::parse(&content, ParserOptions::default()) {
            if let Some(n) = dom
                .query_selector("code")
                .and_then(|mut a| a.next())
                .and_then(|nh| nh.get(dom.parser()))
            {
                debug!("docs.rs source found for {url}");
                return Ok(n.inner_text(dom.parser()).into_owned());
            }
        }
    }
    Ok(content)
}

async fn get_license_texts_from_crates_io_package<'a>(
    package: &Package,
    use_latest: bool,
) -> Result<Vec<String>> {
    let list_url = format!(
        "https://docs.rs/crate/{}/{}/source/",
        package.name,
        if use_latest {
            "latest".into()
        } else {
            package.version.to_string()
        }
    );
    let mut response = surf::get(&list_url).await?;
    if response.status() == StatusCode::NotFound {
        info!("docs.rs does not have {} {}", package.name, package.version);
        return Ok(vec![]);
    }
    let content = response.body_string().await?;
    let dom = tl::parse(&content, ParserOptions::default())?;
    let n = dom
        .query_selector("a")
        .ok_or_else(|| Error::NoPackageMenu(list_url.to_owned()))?;
    let futures = n
        .filter_map(|a| a.get(dom.parser()))
        .filter_map(|a| a.as_tag())
        .filter_map(|a| a.attributes().get("href")?)
        .map(|a| a.as_utf8_str())
        .filter(|a| a.to_lowercase().contains("license") || a.to_lowercase().contains("licence"))
        .filter_map(|a| a.strip_prefix("./").map(|a| a.to_owned()))
        .filter_map(|a| Url::parse(&format!("{list_url}{a}")).ok())
        .inspect(|a| {
            debug!(
                "Found license in {a} for {} {}",
                package.name, package.version
            )
        })
        .map(get_license_text_from_url)
        .collect::<Vec<_>>();
    let texts = future::try_join_all(futures).await?;
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
        let licenses = future::try_join_all(futures).await?;
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
        warn!("No licenses found for {} {}", package.name, package.version)
    }

    Ok(Cow::Owned(licenses))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseRetriever<'a>(Vec<(Package, Cow<'a, [String]>)>);
impl<'a> LicenseRetriever<'a> {
    pub async fn async_from_config(config: &'a Config) -> Result<LicenseRetriever<'a>> {
        let meta = get_metadata(Option::<PathBuf>::None)?;
        let futures = meta
            .packages
            .iter()
            .map(|p| get_license_texts_from_package(p, config))
            .collect::<Vec<_>>();
        let licenses = future::try_join_all(futures)
            .await?
            .into_iter()
            .zip_eq(&meta.packages)
            .map(|(licenses, package)| (package.to_owned(), licenses))
            .collect();
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
