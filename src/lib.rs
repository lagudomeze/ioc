use ioc_core::{Bean, BeanContainer, BeanContainerBuilder, BeanFactory};
use linkme::distributed_slice;
use log::info;

#[derive(Debug)]
pub struct BeanRegistry {
    builder: BeanContainerBuilder,
}

impl BeanRegistry {
    pub fn register<F: BeanFactory + 'static>(&mut self, module: &'static str) {
        let definition = F::T::definition();
        self.builder.append::<F>().unwrap();

        info!(
            "register bean name:{} 
            module:{module} type:{}",
            definition.name, definition.type_name
        );
    }

    pub(crate) fn new() -> Self {
        Self {
            builder: BeanContainer::builder(),
        }
    }
}

#[distributed_slice]
pub static BEAN_COLLECTOR: [fn(&mut BeanRegistry)];

pub fn run_app() {
    let mut ctx = BeanRegistry::new();
    for collect in BEAN_COLLECTOR {
        collect(&mut ctx);
    }

    let _container = ctx.builder.build().expect("error");

    //todo 后续找到container中的 需要run的bean执行，或者

    use std::thread;
    use std::time::Duration;

    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_secs(3));
        });
    });
}

pub use ioc_derive::{run, Bean};
pub use ioc_core::Ref;
