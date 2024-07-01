use std::any::{type_name, TypeId};
use std::collections::{HashSet, VecDeque};
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::sync::OnceLock;

use cfg_rs::FromConfig;
use log::{debug, trace};

use crate::config::Config;
use crate::IocError;

// The `BeanId` struct is used to uniquely identify beans within the IoC container.
// It contains a static string slice as the bean's name and a `TypeId` for the bean's type.
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct BeanId {
    /// The name of the bean, used as a human-readable identifier.
    pub name: &'static str,
    /// The `TypeId` of the bean's type, used to ensure type safety in the container.
    pub type_id: TypeId,
}

// The `BeanSpec` struct defines the specification of a bean, which includes its identifier,
// type name, and a drop function that is called when the bean is being destroyed.
#[derive(Copy, Clone)]
pub struct BeanSpec {
    /// The unique identifier for the bean.
    pub bean_id: BeanId,
    /// The type name of the bean, primarily for debugging purposes.
    pub type_name: &'static str,
    /// The name of the factory that will be used to create the bean.
    pub factory_name: &'static str,
    /// A function that will be called to perform any necessary cleanup when the bean is destroyed.
    pub drop: fn()
}

impl Debug for BeanSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BeanSpec")
            .field("name", &self.bean_id.name)
            .field("type_name", &self.type_name)
            .field("factory_name", &self.factory_name)
            .field("drop", &"function")
            .finish()
    }
}

/// The `Context` struct represents the IoC container's context, managing bean lifecycle, dependencies, and configuration.
/// It contains a list of ready beans, a stack of pending beans, and sets of identifiers for ready and pending beans.
#[derive(Debug)]
pub struct Context {
    /// The configuration settings for the IoC container.
    pub(crate) config: Config,

    /// A list of beans that are ready to be injected into other beans.
    ready_beans: Vec<BeanSpec>,

    /// A set of identifiers for beans that are ready to be injected into other beans.
    ready_bean_ids: HashSet<BeanId>,

    /// A stack of beans that are pending initialization.
    pending_chain: VecDeque<BeanSpec>,
}

/// The `BeanFactory` trait defines the contract for creating an instance of a bean.
#[diagnostic::on_unimplemented(
    message = "BeanFactory is not implemented for this type `{Self}`",
    label = "implement BeanFactory for this type",
)]
pub trait BeanFactory {
    /// The type of bean that will be created by this factory.
    type Bean;

    /// Constructs an instance of the bean using the provided context.
    fn build(ctx: &mut Context) -> crate::Result<Self::Bean>;

    /// Performs any necessary cleanup when the bean is being destroyed.
    fn destroy(_product: &Self::Bean) {}
}

/// The `Bean` trait extends `BeanFactory` with additional functionality for managing bean lifecycle within the IoC container.
#[diagnostic::on_unimplemented(
    message = "Bean is not implemented for this type `{Self}`",
    label = "implement Bean for this type",
    note = " add Bean macro for this type
    #[derive(Bean)]
    struct YourType {{
        ...
    }}
    ",
    note = "or define your bean factory type
    impl BeanFactory for YourFactoryType {{
        type Bean = YourType;

        fn build(ctx: &mut Context) -> ioc::Result<Self::Bean> {{
            Ok(YourType {{ }})
        }}
    }}

    #[derive(Bean)]
    #[custom_factory]
    struct YourFactoryType;
    "
)]
pub trait Bean: BeanFactory<Bean: 'static + Sized> {

    /// Returns a reference to the holder that contains the singleton instance of this bean.
    fn holder<'a>() -> &'a OnceLock<Self::Bean>;

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

    /// Initializes the bean in the context, ensuring it is ready for injection.
    fn init<'a>(ctx: &mut Context) -> crate::Result<&'a Self::Bean>
    where
        Self: Sized,
    {
        ctx.get_or_init::<Self>()
    }

    /// Returns the name of the bean, which is used for identification within the IoC container.
    fn name() -> &'static str {
        Self::bean_type_name()
    }

    /// Returns a unique identifier for the bean based on its type.
    fn bean_id() -> BeanId {
        let name = Self::name();
        let type_id = TypeId::of::<Self::Bean>();
        BeanId {
            name,
            type_id,
        }
    }

    /// Returns the specification for this bean, including its identifier and type name.
    fn spec() -> BeanSpec {
        let bean_id = Self::bean_id();
        let type_name: &str = Self::bean_type_name();
        let factory_name = std::any::type_name::<Self>();
        let spec = BeanSpec {
            bean_id,
            type_name,
            factory_name,
            drop: || {
                if let Some(r) = Self::holder().get() {
                    Self::destroy(r);
                }
            },
        };

        trace!("build spec {:?}", spec);

        spec
    }

    /// Returns the type name of the bean as a static string slice.
    fn bean_type_name<'a>() -> &'a str {
        type_name::<Self::Bean>()
    }
}

// `DropGuard` is responsible for the cleanup logic of the IoC container.
pub struct DropGuard {
    ready_beans: Vec<BeanSpec>,
}

impl Drop for DropGuard {
    /// Automatically performs the cleanup of all registered beans when the `DropGuard` instance is dropped.
    fn drop(&mut self) {
        debug!("Starting cleanup of beans.");
        // Iterate and clean up all beans to ensure resources are properly released.
        for bean_spec in self.ready_beans.iter().rev() {
            debug!("bean {:?} is cleaning", bean_spec);
            // Call the drop function for each bean to perform cleanup.
            (bean_spec.drop)();
        }
        // Perform any other necessary cleanup here.
        debug!("Cleanup of beans completed.");
    }
}

impl Context {

    pub fn new(config: Config) -> Self {
        Self {
            config,
            ready_beans: Default::default(),
            ready_bean_ids: Default::default(),
            pending_chain: Default::default(),
        }
    }

    pub fn get_config<T: FromConfig>(&self, key: &str) -> crate::Result<T> {
        Ok(self.config.source.get(key)?)
    }

    /// Attempts to retrieve or initialize a bean instance of type `B` within the IoC container.
    /// This method ensures that all dependencies are resolved and the bean is ready for use.
    /// It also utilizes a cache to quickly check for circular dependencies, returning an error if one is detected.
    pub fn get_or_init<'a, B: Bean>(&mut self) -> crate::Result<&'a B::Bean> {
        let spec = B::spec();

        // Check if the bean is already initialized and return it if so.
        if self.ready_bean_ids.contains(&spec.bean_id) {
            return B::try_get();
        }

        // Use the cache to detect potential circular dependencies by checking if the bean
        // is currently in the process of being initialized.
        // Check if the bean is currently being initialized and return an error if so.
        for pending_spec in self.pending_chain.iter() {
            if spec.bean_id.eq(&pending_spec.bean_id) {
                return Err(IocError::CircularDependency);
            }
        }
        self.pending_chain.push_back(spec);
        debug!("bean {:?} is pending! ", spec);

        // The holder's `get_or_try_init` method will attempt to build the bean if it's not already initialized.
        let result = B::holder().get_or_try_init(|| B::build(self));

        let ready_bean = self.pending_chain
            .pop_back()
            .expect("Initialization stack is unexpectedly empty");

        if ready_bean.bean_id != spec.bean_id {
            panic!("Initialization stack order corrupted");
        }


        if result.is_ok() {
            self.ready_beans.push(ready_bean);
            self.ready_bean_ids.insert(ready_bean.bean_id);
            debug!("bean {:?} is ready! ", ready_bean);
        }

        result
    }

    pub fn complete(self) -> DropGuard {
        DropGuard {
            ready_beans: self.ready_beans
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
