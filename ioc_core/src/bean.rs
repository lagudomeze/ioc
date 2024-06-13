use std::any::{type_name, TypeId};
use std::collections::{HashSet, VecDeque};
use std::sync::OnceLock;

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

pub trait Bean {
    type Type: 'static + Sized;
    type Factory: BeanFactory<Bean=Self::Type>;

    fn holder<'a>() -> &'a OnceLock<Self::Type>;

    fn try_get<'a>() -> crate::Result<&'a Self::Type> {
        let type_name: &str = type_name::<Self::Type>();
        Self::holder()
            .get()
            .ok_or(IocError::DependNotReady {
                type_name
            })
    }

    fn get<'a>() -> &'a Self::Type {
        //todo
        Self::try_get().expect("")
    }

    fn init<'a>(ctx: &mut Context) -> crate::Result<&'a Self::Type> where Self: Sized {
        ctx.get_or_init::<Self>()
    }

    fn destroy(_product: &Self::Type) {}

    fn name() -> &'static str {
        type_name::<Self::Type>()
    }

    fn bean_id() -> BeanId {
        let name = Self::name();
        let type_id = TypeId::of::<Self::Type>();
        BeanId {
            name,
            type_id,
        }
    }

    fn spec() -> BeanSpec {
        let bean_id = Self::bean_id();
        let type_name: &str = type_name::<Self::Type>();

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

    pub fn init<B>(&mut self, bean: B::Type) -> Option<B::Type> where B: Bean {
        let spec = B::spec();
        if self.ready_bean_ids.contains(&spec.bean_id) {
            None
        } else {
            B::holder().set(bean).err()
        }
    }

    pub fn get_or_init<'a, B: Bean>(&mut self) -> crate::Result<&'a B::Type> {
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
                let result = holder.get_or_try_init(|| B::Factory::build(self));
                self.remove_pending(&spec);
                if result.is_ok() {
                    self.ready_bean_ids.insert(spec.bean_id.clone());
                    self.ready_beans.push(spec);
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

    impl Bean for CfgA {
        type Type = Self;
        type Factory = Self;

        fn holder<'a>() -> &'a OnceLock<Self::Type> {
            static HOLDER: OnceLock<CfgA> = OnceLock::new();
            &HOLDER
        }
    }

    pub struct A(String);

    impl BeanFactory for A {
        type Bean = A;

        fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
            let cfg = ctx.get_or_init::<CfgA>()?;
            Ok(A(cfg.v.clone()))
        }
    }

    impl Bean for A {
        type Type = Self;
        type Factory = Self;

        fn holder<'a>() -> &'a OnceLock<Self> {
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
        type Type = Self;
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

    mod dep {
        use std::sync::OnceLock;

        use cfg_rs::{Configuration, init_cargo_env};

        use crate::{Bean, IocError};
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

        impl BeanFactory for A {
            type Bean = A;

            fn build(_: &mut Context) -> crate::Result<Self::Bean> {
                Ok(A("this is A".to_string()))
            }
        }

        impl Bean for A {
            type Type = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Type> {
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
            type Type = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Type> {
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
            type Type = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Type> {
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
            type Type = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Type> {
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
            type Type = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Type> {
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
            type Type = Self;
            type Factory = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Type> {
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
