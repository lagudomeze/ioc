#![feature(once_cell_try, assert_matches)]

pub use bean::{
    Bean,
    BeanFactory,
    BeanSpec,
    Context
};
pub use config::{
    AppConfigLoader, AppName, Config, ConfigPath, ConfigProfile
};
pub use error::{IocError, Result};

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

