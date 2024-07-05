use std::result::Result as StdResult;
use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: `{0}`")]
    IoError(#[from] io::Error),
    #[error("syn error: `{0}`")]
    SynError(#[from] syn::Error),
    #[error("no parent: `{0}`")]
    NoParent(String),
    #[error("Not found file of `{0}`")]
    FileNotFound(String),
}

pub type Result<T> = StdResult<T, Error>;