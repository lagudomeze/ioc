#![feature(once_cell_try, assert_matches)]

pub use bean::{
    Bean,
    BeanId,
    BeanInfo,
    BeanSpec,
    Construct,
    Destroy,
    DropGuard,
    InitCtx
};
pub use config::{AppConfigLoader, Config};
pub use error::{IocError, Result};
pub use init::{Init, Wrapper};
pub use types::{BeanFamily, Method};

mod bean;
mod error;
mod config;
pub mod types;
mod init;

