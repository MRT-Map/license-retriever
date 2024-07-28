use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    // #[error("http error: {0:?} at {1}")]
    // Http(surf::Error, Url),
    #[error("git error: {0:?}")]
    Git(#[from] git2::Error),
    #[error("cargo metadata error: {0:?}")]
    Metadata(#[from] cargo_metadata::Error),
    // #[error("html parse error: {0:?}")]
    // Html(#[from] tl::ParseError),
    #[error("environment variable error: {0:?}")]
    Env(#[from] std::env::VarError),
    #[error("encoding error: {0:?}")]
    Encode(#[from] rmp_serde::encode::Error),
    #[error("decoding error: {0:?}")]
    Decode(#[from] rmp_serde::decode::Error),
    #[error("i/o error: {0:?}")]
    Io(#[from] std::io::Error),
    // #[error("url parsing error")]
    // Url(#[from] url::ParseError),
    // #[error("`{0} has no file list")]
    // NotCratesIoFileList(Url),
    // #[error("`{0}` has no files, or is not a github repo home page")]
    // NotGithubHomePage(Url),
    #[error("`{0}` specified in `license_copying_crates` not found in package list")]
    CopierCrateNotFound(String),
    #[error("`{0}` specified in `license_copying_crates` not found in package list")]
    CopiedCrateNotFound(String),
    #[error("unknown error")]
    Unknown,
}

// pub trait ErrWithUrl<T> {
//     fn err_with_url(self, err: &Url) -> Result<T>;
// }
//
// impl<T> ErrWithUrl<T> for surf::Result<T> {
//     fn err_with_url(self, err: &Url) -> Result<T> {
//         self.map_err(|e| Error::Http(e, err.to_owned()))
//     }
// }

pub type Result<T, E = Error> = std::result::Result<T, E>;
