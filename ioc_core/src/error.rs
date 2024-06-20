use thiserror::Error;
use std::io;

pub type Result<T> = std::result::Result<T, IocError>;

#[derive(Debug, Error)]
pub enum IocError {
    #[error("fetch config error: `{0}`")]
    ConfigError(String),
    #[error("required {type_name} is not init!")]
    DependNotReady { type_name: &'static str },
    #[error("circular dependency")]
    CircularDependency,
    #[error("io: `{0}`")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<cfg_rs::ConfigError> for IocError {
    fn from(value: cfg_rs::ConfigError) -> Self {
        Self::ConfigError(format!("{value:?}"))
    }
}
