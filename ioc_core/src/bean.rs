use std::{
    any::{self, TypeId},
    fmt::Debug,
    hash::{
        Hash,
        Hasher,
    },
    sync::OnceLock,
};

use crate::{
    InitContext,
    IocError,
};

#[derive(Debug, Eq, Copy, Clone)]
pub struct BeanInfo {
    /// The name of the bean, used as a human-readable name.
    pub(crate) name: &'static str,
    /// The type name of the bean, primarily for debugging purposes.
    pub(crate) bean_type_name: &'static str,
    /// The name of the spec type of unique identify of bean
    pub(crate) spec_name: &'static str,
    /// The `TypeId` of the bean spec's type, used to ensure type safety in the container.
    pub(crate) spec_spec_id: TypeId,
}

impl Hash for BeanInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.spec_spec_id.hash(state);
    }
}
impl PartialEq<Self> for BeanInfo {
    fn eq(&self, other: &Self) -> bool {
        self.spec_spec_id == other.spec_spec_id
    }
}
impl PartialEq<BeanId> for BeanInfo {
    fn eq(&self, other: &BeanId) -> bool {
        self.spec_spec_id == other.0
    }
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct BeanId(TypeId);

#[diagnostic::on_unimplemented(
    message = "Bean is not implemented for this type `{Self}`",
    label = "implement Bean for this type",
    note = "
    add Bean derive for this type
    #[derive(Bean)]
    #[bean(name = \"your bean name\")]
    struct YourType {{
        ...
    }}",
    note = "
    xor define your bean spec type
    #[bean(name = \"your bean name \")]
    impl BeanSpec for YourBeanSpecType {{
        type Bean = YourBeanType;

        fn build(ctx: &mut Context) -> ioc::Result<Self::Bean> {{
            Ok(YourType {{ }})
        }}

        // optional
        fn drop(_: &Self::Bean) {{
        }}
    }}",
)]
pub trait BeanSpec {
    type Bean;

    fn bean_type_name() -> &'static str {
        any::type_name::<Self::Bean>()
    }

    fn spec_type_name() -> &'static str {
        any::type_name::<Self>()
    }

    fn bean_type_id() -> TypeId
    where
        Self::Bean: 'static,
    {
        TypeId::of::<Self::Bean>()
    }

    fn spec_type_id() -> TypeId
    where
        Self: 'static,
    {
        TypeId::of::<Self>()
    }

    fn bean_info() -> BeanInfo
    where
        Self: 'static,
    {
        let name = Self::name();
        let bean_type_name = Self::bean_type_name();
        let spec_name = Self::spec_type_name();
        let spec_spec_id = Self::spec_type_id();
        BeanInfo {
            name,
            bean_type_name,
            spec_name,
            spec_spec_id,
        }
    }

    fn bean_id() -> BeanId
    where
        Self: 'static,
    {
        BeanId(Self::spec_type_id())
    }

    fn build<I>(ctx: &mut I) -> crate::Result<Self::Bean>
    where
        I: InitContext;

    fn holder<'a>() -> &'a OnceLock<Self::Bean>;

    fn name() -> &'static str {
        Self::spec_type_name()
    }

    fn drop(_: &Self::Bean) {}

    /// Attempts to retrieve a reference to the bean instance, returning an error if the bean is not yet ready.
    fn try_get<'a>() -> crate::Result<&'a Self::Bean> {
        Self::holder()
            .get()
            .ok_or(IocError::DependNotReady {
                type_name: Self::bean_type_name()
            })
    }

    fn get<'a>() -> &'a Self::Bean {
        Self::try_get().expect("Failed to get bean from context")
    }
}

#[cfg(test)]
mod tests {
    use cfg_rs::*;

    use crate::{
        BeanSpec,
        InitCtx,
    };

    use super::*;

    #[derive(FromConfig)]
    #[config(prefix = "cfg_a")]
    pub struct CfgA {
        #[config(name = "a")]
        v: String,
        #[config(name = "b")]
        t: String,
    }

    impl BeanSpec for CfgA {
        type Bean = Self;

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<CfgA> = OnceLock::new();
            &HOLDER
        }

        fn build<I>(ctx: &mut I) -> crate::Result<Self::Bean>
        where
            I: InitContext,
        {
            ctx.get_predefined_config()
        }
    }

    pub struct A(String);

    impl BeanSpec for A {
        type Bean = Self;

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<A> = OnceLock::new();
            &HOLDER
        }

        fn build<I>(ctx: &mut I) -> crate::Result<Self::Bean>
        where
            I: InitContext,
        {
            let cfg_a = ctx.get_or_init::<CfgA>()?;

            Ok(Self(cfg_a.v.to_string()))
        }
    }

    struct B(&'static A, String);

    impl BeanSpec for B {
        type Bean = Self;

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<B> = OnceLock::new();
            &HOLDER
        }

        fn build<I>(ctx: &mut I) -> crate::Result<Self::Bean>
        where
            I: InitContext,
        {
            let cfg = ctx.get_or_init::<CfgA>()?;
            let a = ctx.get_or_init::<A>()?;
            Ok(B(a, cfg.t.clone()))
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

        let mut ctx = InitCtx::new(config);

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

        use crate::{
            bean::BeanSpec,
            init::InitContext,
            IocError
        };
        use crate::init::InitCtx;

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

        impl BeanSpec for A {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<A> = OnceLock::new();
                &HOLDER
            }

            fn build<I>(_: &mut I) -> crate::Result<Self::Bean>
            where
                I: InitContext,
            {
                Ok(A("this is A".to_string()))
            }
        }

        impl BeanSpec for B {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<B> = OnceLock::new();
                &HOLDER
            }

            fn build<I>(ctx: &mut I) -> crate::Result<Self::Bean>
            where
                I: InitContext,
            {
                let a = ctx.get_or_init::<A>()?;
                Ok(B(a, "this is B".to_string()))
            }
        }

        impl BeanSpec for C {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<C> = OnceLock::new();
                &HOLDER
            }

            fn build<I>(ctx: &mut I) -> crate::Result<Self::Bean>
            where
                I: InitContext,
            {
                let a = ctx.get_or_init::<A>()?;
                let b = ctx.get_or_init::<B>()?;
                Ok(C(a, b, "this is C".to_string()))
            }
        }

        impl BeanSpec for D {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<D> = OnceLock::new();
                &HOLDER
            }

            fn build<I>(ctx: &mut I) -> crate::Result<Self::Bean>
            where
                I: InitContext,
            {
                let c = ctx.get_or_init::<C>()?;
                let f = ctx.get_or_init::<F>()?;
                Ok(D(c, f, "this is D".to_string()))
            }
        }

        impl BeanSpec for E {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<E> = OnceLock::new();
                &HOLDER
            }

            fn build<I>(ctx: &mut I) -> crate::Result<Self::Bean>
            where
                I: InitContext,
            {
                let c = ctx.get_or_init::<D>()?;
                Ok(E(c, "this is E".to_string()))
            }
        }

        impl BeanSpec for F {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<F> = OnceLock::new();
                &HOLDER
            }

            fn build<I>(ctx: &mut I) -> crate::Result<Self::Bean>
            where
                I: InitContext,
            {
                let e = ctx.get_or_init::<E>()?;
                Ok(F(e, "this is E".to_string()))
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

            let mut ctx = InitCtx::new(config);

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
