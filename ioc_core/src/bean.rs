use std::any::{type_name, TypeId};
use std::collections::{HashSet, VecDeque};
use std::sync::OnceLock;

use cfg_rs::FromConfig;
use log::debug;

use crate::config::Config;
use crate::IocError;

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct BeanId {
    pub name: &'static str,
    pub type_id: TypeId,
}

#[derive(Debug, Copy, Clone)]
pub struct BeanSpec {
    pub bean_id: BeanId,
    pub type_name: &'static str,
    pub drop: fn()
}

pub struct Context {
    pub(crate) config: Config,

    ready_beans: Vec<BeanSpec>,
    pending_bean_stack: VecDeque<BeanSpec>,

    ready_bean_ids: HashSet<BeanId>,
    pending_bean_ids: HashSet<BeanId>,
}

pub trait BeanFactory {
    type Bean;

    fn build(ctx: &mut Context) -> crate::Result<Self::Bean>;
}

pub trait Bean: BeanFactory<Bean: 'static + Sized> {
    fn holder<'a>() -> &'a OnceLock<Self::Bean>;

    fn try_get<'a>() -> crate::Result<&'a Self::Bean> {
        Self::holder()
            .get()
            .ok_or(IocError::DependNotReady {
                type_name: Self::bean_type_name()
            })
    }

    fn get<'a>() -> &'a Self::Bean {
        //todo
        Self::try_get().expect("")
    }

    fn init<'a>(ctx: &mut Context) -> crate::Result<&'a Self::Bean>
    where
        Self: Sized,
    {
        ctx.get_or_init::<Self>()
    }

    fn destroy(_product: &Self::Bean) {}

    fn name() -> &'static str {
        Self::bean_type_name()
    }

    fn bean_id() -> BeanId {
        let name = Self::name();
        let type_id = TypeId::of::<Self::Bean>();
        BeanId {
            name,
            type_id,
        }
    }

    fn spec() -> BeanSpec {
        let bean_id = Self::bean_id();
        let type_name: &str = Self::bean_type_name();

        debug!("name:{} type:{type_name} id:{:?}", bean_id.name, bean_id.type_id);
        BeanSpec {
            bean_id,
            type_name,
            drop: || {
                if let Some(r) = Self::holder().get() {
                    Self::destroy(r);
                }
            },
        }
    }

    fn bean_type_name<'a>() -> &'a str {
        type_name::<Self::Bean>()
    }
}

impl Context {
    fn pending(&mut self, bean_definition: &BeanSpec) {
        self.pending_bean_ids.insert(bean_definition.bean_id.clone());
        self.pending_bean_stack.push_back(bean_definition.clone());
    }

    fn remove_pending(&mut self, bean_definition: &BeanSpec) {
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

    pub fn init<B>(&mut self, bean: B::Bean) -> Option<B::Bean>
    where
        B: Bean,
    {
        let spec = B::spec();
        if self.ready_bean_ids.contains(&spec.bean_id) {
            None
        } else {
            B::holder().set(bean).err()
        }
    }

    pub fn get_or_init<'a, B: Bean>(&mut self) -> crate::Result<&'a B::Bean> {
        let spec = B::spec();
        if self.ready_bean_ids.contains(&spec.bean_id) {
            B::try_get()
        } else {
            if self.pending_bean_ids.contains(&spec.bean_id) {
                //todo log?
                return Err(IocError::CircularDependency);
            } else {
                self.pending(&spec);
                let holder = B::holder();
                let result = holder.get_or_try_init(|| B::build(self));
                self.remove_pending(&spec);
                if result.is_ok() {
                    self.ready_bean_ids.insert(spec.bean_id.clone());
                    self.ready_beans.push(spec);
                }
                return result;
            }
        }
    }

    pub fn get_config<T: FromConfig>(&self, key: &str) -> crate::Result<T> {
        Ok(self.config.source.get(key)?)
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

    impl Bean for CfgA {

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<CfgA> = OnceLock::new();
            &HOLDER
        }
    }

    pub struct A(String);

    impl BeanFactory for A {
        type Bean = Self;

        fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
            let cfg = ctx.get_or_init::<CfgA>()?;
            Ok(A(cfg.v.clone()))
        }
    }

    impl Bean for A {

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<A> = OnceLock::new();
            &HOLDER
        }
    }

    struct B(&'static A, String);

    impl BeanFactory for B {
        type Bean = Self;

        fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
            let cfg = ctx.get_or_init::<CfgA>()?;
            let a = ctx.get_or_init::<A>()?;
            Ok(B(a, cfg.t.clone()))
        }
    }

    impl Bean for B {

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
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

    mod dep {
        use std::assert_matches::assert_matches;
        use std::sync::OnceLock;

        use cfg_rs::{Configuration, init_cargo_env};

        use crate::{Bean, IocError};
        use crate::bean::{BeanFactory, Context};

        #[derive(Debug)]
        pub struct A(String);

        #[derive(Debug)]
        pub struct B(&'static A, String);

        #[derive(Debug)]
        pub struct C(&'static A, &'static B, String);
        #[allow(dead_code)]
        #[derive(Debug)]
        pub struct D(&'static C, &'static F, String);
        #[allow(dead_code)]
        #[derive(Debug)]
        pub struct E(&'static D, String);
        #[allow(dead_code)]
        #[derive(Debug)]
        pub struct F(&'static E, String);

        impl BeanFactory for A {
            type Bean = Self;

            fn build(_: &mut Context) -> crate::Result<Self::Bean> {
                Ok(A("this is A".to_string()))
            }
        }

        impl Bean for A {

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<A> = OnceLock::new();
                &HOLDER
            }
        }


        impl BeanFactory for B {
            type Bean = B;

            fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
                let a = ctx.get_or_init::<A>()?;
                Ok(B(a, "this is B".to_string()))
            }
        }

        impl Bean for B {

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<B> = OnceLock::new();
                &HOLDER
            }
        }

        impl BeanFactory for C {
            type Bean = Self;

            fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
                let a = ctx.get_or_init::<A>()?;
                let b = ctx.get_or_init::<B>()?;
                Ok(C(a, b, "this is C".to_string()))
            }
        }

        impl Bean for C {

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<C> = OnceLock::new();
                &HOLDER
            }
        }

        impl BeanFactory for D {
            type Bean = Self;

            fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
                let c = ctx.get_or_init::<C>()?;
                let f = ctx.get_or_init::<F>()?;
                Ok(D(c, f, "this is D".to_string()))
            }
        }

        impl Bean for D {

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<D> = OnceLock::new();
                &HOLDER
            }
        }

        impl BeanFactory for E {
            type Bean = Self;

            fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
                let c = ctx.get_or_init::<D>()?;
                Ok(E(c, "this is E".to_string()))
            }
        }

        impl Bean for E {

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<E> = OnceLock::new();
                &HOLDER
            }
        }

        impl BeanFactory for F {
            type Bean = Self;

            fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
                let e = ctx.get_or_init::<E>()?;
                Ok(F(e, "this is E".to_string()))
            }
        }

        impl Bean for F {

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

            assert_matches!(ctx.get_or_init::<E>(), Err(IocError::CircularDependency));
            assert_matches!(ctx.get_or_init::<F>(), Err(IocError::CircularDependency));
            assert_matches!(ctx.get_or_init::<D>(), Err(IocError::CircularDependency));

            Ok(())
        }
    }
}
