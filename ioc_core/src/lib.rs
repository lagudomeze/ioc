#![feature(strict_provenance, exposed_provenance, new_uninit)]

mod error;

pub(crate) mod bean;
pub(crate) mod container;

pub use bean::{Bean, BeanDefinition, BeanQuery};
pub use container::{BeanContainer, BeanContainerBuilder, BeanFactory, BeanId, BeanRetriever, Ref, ContainerError};
