#![feature(strict_provenance, exposed_provenance, offset_of, new_uninit)]

pub use log::*;
pub mod error;

pub(crate) mod bean;
pub(crate) mod container;

pub use bean::{Bean, BeanDefinition, BeanQuery};
pub use container::{BeanId, BeanContainer, BeanContainerBuilder, BeanFactory, BeanRetriever, Ref};