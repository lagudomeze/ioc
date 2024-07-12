#![feature(once_cell_try, assert_matches)]

pub use bean::{
    BeanId,
    BeanInfo,
    BeanSpec,
};
pub use config::{AppConfigLoader, Config};
pub use error::{IocError, Result};
pub use init::{Init, Wrapper, InitCtx, InitContext};
pub use types::{BeanFamily, Method};

mod bean;
mod error;
mod config;
pub mod types;
mod init;
mod bootstrap;

