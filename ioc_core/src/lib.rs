#![feature(strict_provenance, exposed_provenance, offset_of)]
use std::sync::OnceLock;

pub use log::*;
pub mod error;

static GLOBAL_CONTEXT: OnceLock<ApplicationContext> = OnceLock::new();

pub(crate) mod bean;
mod container;
mod container2;

pub use bean::{Bean, BeanDefinition};
pub use container::{BeanContainer, BeanContainerBuilder};

pub struct ApplicationContext {
    container: BeanContainer,
}

impl ApplicationContext {
    fn new(builder: BeanContainerBuilder) -> Self {
        let container = builder.build();

        Self { container }
    }
}
