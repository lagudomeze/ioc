use thiserror::Error;

#[derive(Error, Debug)]
pub enum IocError {
    #[error("No bean of type: {type_name}")]
    NotRegisteredBean { type_name: &'static str },
    #[error("loop dependency")]
    LoopDependency,
    #[error("unknown error")]
    Unknown,
}
