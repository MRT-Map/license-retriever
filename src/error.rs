use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("git error: {0:?}")]
    Git(#[from] git2::Error),
    #[error("cargo metadata error: {0:?}")]
    Metadata(#[from] cargo_metadata::Error),
    #[error("environment variable error: {0:?}")]
    Env(#[from] std::env::VarError),
    #[error("encoding error: {0:?}")]
    Encode(#[from] rmp_serde::encode::Error),
    #[error("decoding error: {0:?}")]
    Decode(#[from] rmp_serde::decode::Error),
    #[error("i/o error: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("No licenses found for: {0}")]
    NoLicensesFound(String),
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
