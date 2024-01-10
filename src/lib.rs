#![feature(error_generic_member_access)]

use ioc_core::{Bean, BeanContainer, BeanContainerBuilder, BeanFactory, ContainerError};
use linkme::distributed_slice;
use log::info;

#[derive(Debug)]
pub struct BeanRegistry {
    builder: BeanContainerBuilder,
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IocError {
    #[error("container error")]
    ContainerError {
        #[backtrace]
        #[from]
        source: ContainerError,
    },
}

type Result<T> = std::result::Result<T, IocError>;

impl BeanRegistry {
    pub fn register<F: BeanFactory + 'static>(&mut self, module: &'static str) {
        let type_name = F::T::type_name();
        let name = F::T::name();

        self.builder.append::<F>().expect(&format!(
            "register bean name:{name} module:{module} type:{type_name} failed"
        ));

        info!("register bean name:{name} module:{module} type:{type_name}");
    }

    pub(crate) fn new() -> Self {
        Self {
            builder: BeanContainer::builder(),
        }
    }
}

#[distributed_slice]
pub static BEAN_COLLECTOR: [fn(&mut BeanRegistry)];

pub fn run_app() -> Result<()> {
    env_logger::init();

    let mut ctx = BeanRegistry::new();
    for collect in BEAN_COLLECTOR {
        collect(&mut ctx);
    }

    let _container = ctx.builder.build()?;

    //todo 后续找到container中的 需要run的bean执行，或者

    use std::thread;
    use std::time::Duration;

    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_secs(3));
        });
    });

    Ok(())
}

pub use ioc_core::Ref;
pub use ioc_derive::{run, Bean};
