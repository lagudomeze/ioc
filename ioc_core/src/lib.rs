#![feature(once_cell_try)]

pub use bean::{
    Bean,
    BeanDefinition,
    BeanHolder,
};
pub use error::{IocError, Result};
pub use init::BeanId;

pub(crate) mod bean;

mod error;
mod init;
mod config;

