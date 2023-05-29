use std::{borrow::Cow, collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use futures::future;
use itertools::Itertools;
use lazy_regex::{lazy_regex, Lazy, Regex};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use surf::{StatusCode, Url};
use tl::ParserOptions;

pub static DOCS_RS_REGEX: Lazy<Regex> = lazy_regex!(r"^https?://docs\.rs/crate/.*?/.*?/source");

#[derive(Clone, Default)]
pub struct Config {
    pub license_text_overrides: HashMap<String, Vec<String>>,
    pub license_url_overrides: HashMap<String, Vec<Url>>,
    pub manifest_path: Option<PathBuf>,
}
impl Config {
    pub fn manifest_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.manifest_path = Some(path.into());
        self
    }
    pub fn override_license_text(
        &mut self,
        crate_name: impl Into<String>,
        licenses: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.license_text_overrides.insert(
            crate_name.into(),
            licenses.into_iter().map(|a| a.into()).collect(),
        );
        self
    }
    pub fn override_license_url(
        &mut self,
        crate_name: impl Into<String>,
        urls: impl IntoIterator<Item = impl Into<Url>>,
    ) -> &mut Self {
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

async fn get_license_text_from_url(url: Url) -> surf::Result<String> {
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
) -> surf::Result<Vec<String>> {
    let list_url = format!(
        "https://docs.rs/crate/{}/{}/source/",
        package.name,
        package.version.to_string()
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
        .ok_or_else(|| anyhow!("No package menu"))?;
    let futures = n
        .filter_map(|a| a.get(dom.parser()))
        .filter_map(|a| a.as_tag())
        .filter_map(|a| a.attributes().get("href")?)
        .map(|a| a.as_utf8_str())
        .filter(|a| a.contains("LICENSE"))
        .filter_map(|a| a.strip_prefix("./").map(|a| a.to_owned())) // todo
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
) -> surf::Result<Cow<'a, [String]>> {
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
        "Attempting to license for {} {} from crates.io",
        package.name, package.version
    );
    let licenses = get_license_texts_from_crates_io_package(package).await?;

    if licenses.is_empty() {
        warn!("No licenses found for {} {}", package.name, package.version)
    }

    Ok(Cow::Owned(licenses))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseRetriever<'a>(HashMap<String, Cow<'a, [String]>>);
impl<'a> LicenseRetriever<'a> {
    pub async fn from_config(config: &'a Config) -> surf::Result<LicenseRetriever<'a>> {
        let meta = get_metadata(Option::<PathBuf>::None).unwrap();
        let futures = meta
            .packages
            .iter()
            .map(|p| get_license_texts_from_package(p, config))
            .collect::<Vec<_>>();
        let licenses = future::try_join_all(futures)
            .await?
            .into_iter()
            .zip_eq(&meta.packages)
            .map(|(licenses, package)| (package.name.to_owned(), licenses))
            .collect();
        Ok(LicenseRetriever(licenses))
    }
    pub fn save_to_bytes(&self) -> surf::Result<Vec<u8>> {
        Ok(rmp_serde::to_vec(&self.0)?)
    }
    pub async fn save_in_out_dir(&self, file_name: &str) -> surf::Result<()> {
        async_fs::write(
            PathBuf::try_from(std::env::var("OUT_DIR")?)?.join(file_name),
            self.save_to_bytes()?,
        )
        .await?;
        Ok(())
    }
    pub fn load_from_bytes(bytes: &[u8]) -> surf::Result<LicenseRetriever<'static>> {
        Ok(LicenseRetriever(rmp_serde::from_slice(bytes)?))
    }
}

#[cfg(test)]
mod tests {

    use futures::executor;
    use log::LevelFilter;
    use test_log::test;

    use crate::{Config, LicenseRetriever};

    #[test]
    fn test() {
        log::set_max_level(LevelFilter::Warn);
        let config = Config::default();
        let lr = executor::block_on(LicenseRetriever::from_config(&config)).unwrap();
        assert_eq!(
            lr,
            LicenseRetriever::load_from_bytes(&lr.save_to_bytes().unwrap()).unwrap()
        );
    }
}
