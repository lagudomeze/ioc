use ioc_core::{Bean, BeanContainer, BeanDefinition, ContainerInfo};
use linkme::distributed_slice;
use log::info;

#[derive(Debug)]
pub struct BeanDefinitionCollector {
    bean_definitions: Vec<BeanDefinition>,
}

impl BeanDefinitionCollector {
    pub fn register<B: Bean + 'static>(&mut self, module: &'static str) {
        let definition = B::definition();

        info!(
            "register bean name:{} 
            module:{module} type:{}",
            definition.name, definition.type_name
        );
        self.bean_definitions.push(definition);
    }

    pub(crate) fn new() -> Self {
        Self {
            bean_definitions: vec![],
        }
    }
}

#[distributed_slice]
pub(crate) static BEAN_COLLECTOR: [fn(&mut BeanDefinitionCollector)];

pub fn run_app() {
    let mut ctx = BeanDefinitionCollector::new();
    for collect in BEAN_COLLECTOR {
        collect(&mut ctx);
    }

    let info: ContainerInfo = ContainerInfo::new(ctx.bean_definitions).expect("");

    let _container = BeanContainer::new(info);

    //todo 后续找到container中的 需要run的bean执行，或者

    use std::thread;
    use std::time::Duration;

    thread::scope(|s| {
        println!("aaaa");
        s.spawn(|| {
            thread::sleep(Duration::from_secs(3));
        });
        println!("bbb");
    });
}

#[cfg(test)]
mod tests {
    use ioc_core::BeanId;

    use super::*;

    struct A;

    impl Bean for A {}

    #[distributed_slice(BEAN_COLLECTOR)]
    fn register_bean_a(ctx: &mut BeanDefinitionCollector) {
        ctx.register::<A>(module_path!());
    }

    #[test]
    fn it_works() {

        let mut ctx = BeanDefinitionCollector::new();
        for collect in BEAN_COLLECTOR {
            collect(&mut ctx);
        }
        println!("{:?}", ctx);

        let info: ContainerInfo = ContainerInfo::new(ctx.bean_definitions).expect("");
        let id = info.find::<A>().expect("");
        assert_eq!(BeanId::new(0), id);

        let _container = BeanContainer::new(info);
    }
}
