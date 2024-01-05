#![feature(strict_provenance, exposed_provenance, offset_of, ptr_as_uninit)]

pub use log::*;
pub mod error;

pub(crate) mod bean;
pub(crate) mod container;
pub(crate) mod context;

pub use bean::{Bean, BeanDefinition};
pub use container::{BeanId, ContainerInfo, Ref, BeanContainer};
pub use context::{BeanFactory};