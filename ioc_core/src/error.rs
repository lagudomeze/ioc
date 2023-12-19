
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IocError {
    #[error("No bean of type: {type_name}")]
    NotFound{type_name: &'static str},
    #[error("unknown error")]
    Unknown,
}