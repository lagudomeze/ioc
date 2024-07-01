#![feature(once_cell_try, assert_matches)]

pub use bean::{
    Bean,
    BeanFactory,
    BeanSpec,
    Context,
    DropGuard
};
pub use config::{
    AppConfigLoader, Config,
};
pub use error::{IocError, Result};
pub use init::{Wrapper, Init};
pub use types::{BeanFamily, MethodType};

#[macro_export]
macro_rules! load_config {
    ($($field:ident = $value:expr),* $(,)?) => {
        AppConfigLoader {
            $(
                $field: $value,
            )*
            ..Default::default()
        }.load()
    }
 }

mod bean;
mod error;
mod config;
pub mod types;
mod init;

