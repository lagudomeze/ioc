#![feature(once_cell_try, assert_matches)]

pub use bean::{
    Bean,
    BeanFactory,
    BeanSpec,
    Context,
    DropGuard
};
pub use config::{AppConfigLoader, Config};
pub use error::{IocError, Result};
pub use init::{Init, Wrapper};
pub use types::{BeanFamily, MethodType};

mod bean;
mod error;
mod config;
pub mod types;
mod init;

