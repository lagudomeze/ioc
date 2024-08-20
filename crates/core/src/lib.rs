#![feature(once_cell_try, assert_matches)]

pub use bean::{BeanId, BeanInfo, BeanSpec};
pub use config::{AppConfigLoader, Config};
pub use error::{IocError, Result};
pub use init::{Init, InitContext, InitCtx, Wrapper};
pub use types::{BeanFamily, Method};

mod bean;
mod bootstrap;
mod config;
mod error;
mod init;
pub mod types;
