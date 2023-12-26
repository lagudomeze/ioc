#![feature(strict_provenance, exposed_provenance, offset_of, ptr_as_uninit)]
use std::sync::OnceLock;

pub use log::*;
pub mod error;

pub(crate) mod bean;
mod container;

pub use bean::{Bean, BeanDefinition};
