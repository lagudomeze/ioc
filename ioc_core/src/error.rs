use thiserror::Error;

pub type Result<T> = std::result::Result<T, IocError>;

#[derive(Debug, Error)]
pub enum IocError {
    #[error("fetch config error: `{0}`")]
    ConfigError(String)
}

impl From<cfg_rs::ConfigError> for IocError {
    fn from(value: cfg_rs::ConfigError) -> Self {
        Self::ConfigError(
            format!("{value:?}")
        )
    }
}