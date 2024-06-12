use std::any::{type_name, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::OnceLock;

use cfg_rs::Configuration;
use log::{debug, info, warn};

use crate::IocError;

pub trait Bean {
    fn name() -> &'static str {
        type_name::<Self>()
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct BeanId {
    pub name: &'static str,
    pub type_id: TypeId,
}

#[derive(Debug)]
pub struct BeanDefinition {
    pub bean_id: BeanId,
    pub type_name: &'static str,
}

pub fn definition<B>() -> BeanDefinition where B: 'static + Bean {
    let name = B::name();
    let type_id = TypeId::of::<B>();
    let bean_id = BeanId {
        type_id,
        name,
    };
    let type_name = type_name::<B>();

    debug!(" name:{name} type:{type_name} id:{type_id:?}");

    BeanDefinition {
        bean_id,
        type_name,
    }
}

pub struct Context {
    ready_beans: Vec<BeanDefinition>,
    pending_bean_stack: VecDeque<BeanDefinition>,

    ready_bean_ids: HashSet<BeanId>,
    pending_bean_ids: HashSet<BeanId>,
}

pub trait BeanFactory {
    type Bean: Bean + Sized;

    fn build(self, _: &mut Context) -> crate::Result<Self::Bean>;
}

pub trait BeanHolder {
    type Factory: BeanFactory;
    type Bean = Self::Factory::Bean;

    fn holder<'a>() -> &'a OnceLock<Self::Bean>;

    fn try_get<'a>() -> crate::Result<&'a Self::Bean> {
        let type_name: &str = type_name::<Self::Bean>();
        Self::holder()
            .get()
            .ok_or(IocError::DependNotReady {
                type_name
            })
    }

    fn get<'a>() -> &'a Self::Bean {
        //todo
        Self::try_get().expect("")
    }

    fn init<'a>(ctx: &mut Context) -> crate::Result<&'a Self::Bean> where Self::Bean: 'static {
        ctx.init::<Self>()
    }

    fn destroy(_product: &Self::Bean) {}
}

impl Context {
    fn pending(&mut self, bean_definition: &BeanDefinition) {
        self.pending_bean_ids.push_back(bean_definition.bean_id.clone());
        self.pending_bean_stack.push_back(bean_definition.clone());
    }

    fn remove_pending(&mut self, bean_definition: &BeanDefinition) {
        if let Some(last) = self.pending_bean_stack.pop_back() {
            if bean_definition.eq(&last) {
                self.pending_bean_ids.remove(&last.bean_id);
            } else {
                panic!("some fatal error");
            }
        } else {
            panic!("some fatal error");
        }
    }

    pub fn new() -> Self {
        Self {
            ready_beans: Default::default(),
            pending_bean_stack: Default::default(),
            ready_bean_ids: Default::default(),
            pending_bean_ids: Default::default(),
        }
    }

    pub fn init<'a, H>(&mut self) -> crate::Result<&'a H::Bean>
        where
            H: BeanHolder,
            H::Bean: 'static {
        let definition = definition::<H::Bean>();
        if self.ready_bean_ids.contains(&definition.bean_id) {
            H::try_get()
        } else {
            if self.pending_bean_ids.contains(&definition.bean_id) {
                //todo log?
                return Err(IocError::CircularDependency);
            } else {
                self.pending(&definition);
                let holder = H::holder();
                let result = holder.get_or_try_init(|| H::build(self));
                self.remove_pending(&definition);
                if result.is_ok() {
                    self.ready_bean_ids.insert(definition.bean_id.clone());
                    self.ready_beans.push(definition);
                }
                return result;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use cfg_rs::*;

    use super::*;

    #[derive(FromConfig)]
    #[config(prefix = "cfg_a")]
    pub struct CfgA {
        #[config(name = "a")]
        v: String,
        #[config(name = "b")]
        t: String,
    }

    impl BeanHolder for CfgA {
        type Factory = CfgA;

        fn holder<'a>() -> &'a OnceLock<Self::Product> {
            static HOLDER: OnceLock<CfgA> = OnceLock::new();
            &HOLDER
        }
    }

    pub struct A(String);

    impl Bean for A {}

    pub struct AFactory;

    impl BeanFactory for AFactory {
        type Bean = A;

        fn build(self, ctx: &mut Context) -> crate::Result<Self::Bean> {
            let cfg = ctx.init::<CfgA>()?;
            Ok(A(cfg.v))
        }
    }

    impl BeanHolder for A {
        type Factory = AFactory;

        fn holder<'a>() -> &'a OnceLock<Self> {
            static HOLDER: OnceLock<A> = OnceLock::new();
            &HOLDER
        }
    }

    struct B(&'static A, String);

    impl Bean for B {
    }

    impl BeanFactory for B {
        type Bean = B;

        fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
            let cfg = ctx.init::<CfgA>()?;
            let a = ctx.init::<A>()?;
            Ok(B(a, cfg.v))
        }
    }

    impl BeanHolder for B {
        type Factory = B;

        fn holder<'a>() -> &'a OnceLock<Self> {
            static HOLDER: OnceLock<B> = OnceLock::new();
            &HOLDER
        }
    }

    impl BeanFactory for Configuration {
        type Bean = Configuration;

        fn build(_: &mut Context) -> crate::Result<Self::Bean> {
            init_cargo_env!();

            let config = Configuration::with_predefined_builder()
                .set("cfg_a.a", "data")
                .set("cfg_a.b", "babel")
                .init()?;

            Ok(config)
        }
    }

    impl BeanHolder for Configuration {
        type Factory = Configuration;

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<Configuration> = OnceLock::new();
            &HOLDER
        }
    }

    #[test]
    fn test_inject_init() -> Result<(), ConfigError> {

        let mut ctx = Context::new();

        let a = ctx.init::<A>()?;
        let b = ctx.init::<B>()?;

        assert_eq!("data", a.0);

        assert_eq!("babel", b.1);
        assert_eq!(a as *const A, b.0 as *const A);

        assert_eq!(a as *const A, A::get() as *const A);
        assert_eq!(b as *const B, B::get() as *const B);
    }

    #[test]
    fn it_works() {
        let definition = definition::<A>();
        assert_eq!(definition.bean_id.name, "ioc_core::bean::tests::A");
        assert_eq!(definition.bean_id.type_id, TypeId::of::<A>());
    }
}
