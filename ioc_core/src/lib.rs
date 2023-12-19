use std::sync::OnceLock;

pub use log::*;
pub mod error;

static GLOBAL_CONTEXT: OnceLock<ApplicationContext> = OnceLock::new();

mod bean;
mod container;

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
