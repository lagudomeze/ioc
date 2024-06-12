#![feature(once_cell_try, trait_alias)]
#![feature(associated_type_defaults)]

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

