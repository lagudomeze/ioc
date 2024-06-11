#![feature(once_cell_try, trait_alias)]

pub use bean::{
    Bean,
    BeanDefinition,
    BeanQuery,
    Factory,
    BeanSingleton,
};
pub use error::{IocError, Result};
pub use init::BeanId;

pub(crate) mod bean;

mod error;
mod init;
mod config;

