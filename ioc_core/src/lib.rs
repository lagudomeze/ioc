#![feature(once_cell_try, assert_matches)]

pub use bean::{
    Bean,
    BeanSpec,
    Context,
    BeanFactory
};
pub use config::{
    Config
};
pub use error::{IocError, Result};

mod bean;
mod error;
mod config;

