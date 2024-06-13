#![feature(once_cell_try)]

pub use bean::{
    Bean,
    BeanSpec,
};
pub use error::{IocError, Result};
pub use init::BeanId;

pub(crate) mod bean;

mod error;
mod init;
mod config;

