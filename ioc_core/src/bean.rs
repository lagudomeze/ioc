use std::any::{type_name, TypeId};
use std::collections::{HashSet, VecDeque};
use std::sync::OnceLock;

use log::debug;

use crate::config::Config;
use crate::IocError;

pub trait Bean {
    fn name() -> &'static str {
        type_name::<Self>()
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct BeanId {
    pub name: &'static str,
    pub type_id: TypeId,
}

#[derive(Debug, Copy, Clone)]
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
    pub(crate) config: Config,

    ready_beans: Vec<BeanDefinition>,
    pending_bean_stack: VecDeque<BeanDefinition>,

    ready_bean_ids: HashSet<BeanId>,
    pending_bean_ids: HashSet<BeanId>,
}

pub trait BeanFactory {
    type Bean: Bean + Sized;

    fn build(ctx: &mut Context) -> crate::Result<Self::Bean>;
}

pub struct NeverFactory<T>(std::marker::PhantomData<T>);

impl<T> BeanFactory for NeverFactory<T> where T: Bean + Sized {
    type Bean = T;

    fn build(_: &mut Context) -> crate::Result<Self::Bean> {
        panic!("Your init it direct by Context's init method!")
    }
}

impl<T> NeverFactory<T> {
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

pub trait BeanHolder {
    type Bean: Bean;
    type Factory: BeanFactory<Bean=Self::Bean>;

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

    fn init<'a>(ctx: &mut Context) -> crate::Result<&'a Self::Bean>
        where
            Self: Sized,
            Self::Bean: 'static {
        ctx.get_or_init::<Self>()
    }

    fn destroy(_product: &Self::Bean) {}
}

impl Context {
    fn pending(&mut self, bean_definition: &BeanDefinition) {
        self.pending_bean_ids.insert(bean_definition.bean_id.clone());
        self.pending_bean_stack.push_back(bean_definition.clone());
    }

    fn remove_pending(&mut self, bean_definition: &BeanDefinition) {
        if let Some(last) = self.pending_bean_stack.pop_back() {
            if bean_definition.bean_id.eq(&last.bean_id) {
                self.pending_bean_ids.remove(&last.bean_id);
            } else {
                panic!("some fatal error");
            }
        } else {
            panic!("some fatal error");
        }
    }

    pub fn new(config: Config) -> Self {
        Self {
            config,
            ready_beans: Default::default(),
            pending_bean_stack: Default::default(),
            ready_bean_ids: Default::default(),
            pending_bean_ids: Default::default(),
        }
    }

    pub fn init<H>(&mut self, bean: H::Bean) -> Option<H::Bean>
        where
            H: BeanHolder,
            H::Bean: 'static {
        let definition = definition::<H::Bean>();
        if self.ready_bean_ids.contains(&definition.bean_id) {
            None
        } else {
            H::holder().set(bean).err()
        }
    }

    pub fn get_or_init<'a, H>(&mut self) -> crate::Result<&'a H::Bean>
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
                let result = holder.get_or_try_init(|| H::Factory::build(self));
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

    impl Bean for CfgA {}

    impl BeanHolder for CfgA {
        type Bean = Self;
        type Factory = Self;

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<CfgA> = OnceLock::new();
            &HOLDER
        }
    }

    pub struct A(String);

    impl Bean for A {}

    impl BeanFactory for A {
        type Bean = A;

        fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
            let cfg = ctx.get_or_init::<CfgA>()?;
            Ok(A(cfg.v.clone()))
        }
    }

    impl BeanHolder for A {
        type Bean = Self;
        type Factory = Self;

        fn holder<'a>() -> &'a OnceLock<Self> {
            static HOLDER: OnceLock<A> = OnceLock::new();
            &HOLDER
        }
    }

    struct B(&'static A, String);

    impl Bean for B {
    }

    impl BeanFactory for B {
        type Bean = Self;

        fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
            let cfg = ctx.get_or_init::<CfgA>()?;
            let a = ctx.get_or_init::<A>()?;
            Ok(B(a, cfg.t.clone()))
        }
    }

    impl BeanHolder for B {
        type Bean = Self;
        type Factory = Self;

        fn holder<'a>() -> &'a OnceLock<Self> {
            static HOLDER: OnceLock<B> = OnceLock::new();
            &HOLDER
        }
    }

    #[test]
    fn test_inject_init() -> Result<(), ConfigError> {
        init_cargo_env!();

        let config = Configuration::with_predefined_builder()
            .set("cfg_a.a", "data")
            .set("cfg_a.b", "babel")
            .init()?
            .into();

        let mut ctx = Context::new(config);

        let a = ctx.get_or_init::<A>()?;
        let b = ctx.get_or_init::<B>()?;

        assert_eq!("data", a.0);

        assert_eq!("babel", b.1);
        assert_eq!(a as *const A, b.0 as *const A);

        assert_eq!(a as *const A, A::get() as *const A);
        assert_eq!(b as *const B, B::get() as *const B);

        Ok(())
    }

    #[test]
    fn it_works() {
        let definition = definition::<A>();
        assert_eq!(definition.bean_id.name, "ioc_core::bean::tests::A");
        assert_eq!(definition.bean_id.type_id, TypeId::of::<A>());
    }

    mod dep {
        use std::sync::OnceLock;
        use cfg_rs::{Configuration, init_cargo_env};

        use crate::{Bean, BeanHolder, IocError};
        use crate::bean::{BeanFactory, Context};

        pub struct A(String);

        pub struct B(&'static A, String);

        pub struct C(&'static A, &'static B, String);
        #[allow(dead_code)]
        pub struct D(&'static C, &'static F, String);
        #[allow(dead_code)]
        pub struct E(&'static D, String);
        #[allow(dead_code)]
        pub struct F(&'static E, String);

        impl Bean for A {}

        impl BeanFactory for A {
            type Bean = A;

            fn build(_: &mut Context) -> crate::Result<Self::Bean> {
                Ok(A("this is A".to_string()))
            }
        }

        impl BeanHolder for A {
            type Bean = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<A> = OnceLock::new();
                &HOLDER
            }
        }

        impl Bean for B {}

        impl BeanFactory for B {
            type Bean = B;

            fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
                let a = ctx.get_or_init::<A>()?;
                Ok(B(a, "this is B".to_string()))
            }
        }

        impl BeanHolder for B {
            type Bean = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<B> = OnceLock::new();
                &HOLDER
            }
        }

        impl Bean for C {

        }

        impl BeanFactory for C {
            type Bean = Self;

            fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
                let a = ctx.get_or_init::<A>()?;
                let b = ctx.get_or_init::<B>()?;
                Ok(C(a, b, "this is C".to_string()))
            }
        }

        impl BeanHolder for C {
            type Bean = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<C> = OnceLock::new();
                &HOLDER
            }
        }

        impl Bean for D {

        }

        impl BeanFactory for D {
            type Bean = Self;

            fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
                let c = ctx.get_or_init::<C>()?;
                let f = ctx.get_or_init::<F>()?;
                Ok(D(c, f, "this is D".to_string()))
            }
        }

        impl BeanHolder for D {
            type Bean = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<D> = OnceLock::new();
                &HOLDER
            }
        }

        impl Bean for E {

        }

        impl BeanFactory for E {
            type Bean = Self;

            fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
                let c = ctx.get_or_init::<D>()?;
                Ok(E(c, "this is E".to_string()))
            }
        }

        impl BeanHolder for E {
            type Bean = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<E> = OnceLock::new();
                &HOLDER
            }
        }

        impl Bean for F {

        }

        impl BeanFactory for F {
            type Bean = Self;

            fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
                let e = ctx.get_or_init::<E>()?;
                Ok(F(e, "this is E".to_string()))
            }
        }

        impl BeanHolder for F {
            type Bean = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<F> = OnceLock::new();
                &HOLDER
            }
        }

        #[test]
        fn it_works() -> crate::Result<()> {
            init_cargo_env!();

            let config = Configuration::with_predefined_builder()
                .set("cfg_a.a", "data")
                .set("cfg_a.b", "babel")
                .init()?
                .into();

            let mut ctx = Context::new(config);

            let a = ctx.get_or_init::<A>()?;
            let b = ctx.get_or_init::<B>()?;
            let c = ctx.get_or_init::<C>()?;
            assert_eq!(a as *const A, b.0 as *const A);
            assert_eq!(a as *const A, c.0 as *const A);
            assert_eq!(b as *const B, c.1 as *const B);
            assert_eq!("this is A", &a.0);
            assert_eq!("this is B", &b.1);
            assert_eq!("this is C", &c.2);

            assert_eq!(Some(IocError::CircularDependency), ctx.get_or_init::<E>().err());
            assert_eq!(Some(IocError::CircularDependency), ctx.get_or_init::<F>().err());
            assert_eq!(Some(IocError::CircularDependency), ctx.get_or_init::<D>().err());

            Ok(())
        }
    }
}
