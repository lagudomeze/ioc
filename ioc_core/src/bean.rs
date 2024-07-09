use std::{
    any::{self, TypeId},
    collections::{HashSet, VecDeque},
    fmt::Debug,
    hash::{
        Hash,
        Hasher,
    },
    sync::OnceLock,
};

use cfg_rs::FromConfig;
use log::{debug, trace};

use crate::{
    config::Config,
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

pub trait BeanSpec {
    type Bean;

    fn name() -> &'static str {
        Self::spec_type_name()
    }

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

    fn holder<'a>() -> &'a OnceLock<Self::Bean>;

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
}

/// The `Context` struct represents the IoC container's context, managing bean lifecycle, dependencies, and configuration.
/// It contains a list of ready beans, a stack of pending beans, and sets of identifiers for ready and pending beans.
#[derive(Debug)]
pub struct InitCtx {
    /// The configuration settings for the IoC container.
    pub(crate) config: Config,

    /// A list of beans that are ready to be injected into other beans.
    ready_beans: Vec<(BeanInfo, fn())>,

    /// A set of identifiers for beans that are ready to be injected into other beans.
    ready_bean_ids: HashSet<BeanId>,

    /// A stack of beans that are pending initialization.
    pending_chain: VecDeque<BeanInfo>,
}

#[diagnostic::on_unimplemented(
    message = "Construct is not implemented for this type `{Self}`",
    label = "implement Construct for this type or use Bean derive macro",
)]
pub trait Construct {
    type Bean;

    fn build(ctx: &mut InitCtx) -> crate::Result<Self::Bean>;
}

#[diagnostic::on_unimplemented(
    message = "Destroy is not implemented for this type `{Self}`",
    label = "implement Destroy for this type or use Bean derive macro",
)]
pub trait Destroy {
    type Bean;

    fn drop(_: &Self::Bean) {}
}

/// The `Bean` trait extends `BeanFactory` with additional functionality for managing bean lifecycle within the IoC container.
#[diagnostic::on_unimplemented(
    message = "Bean is not implemented for this type `{Self}`",
    label = "implement Bean for this type",
    note = " add Bean derive for this type
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
pub trait Bean {
    type Construct: Construct;
    type Spec: BeanSpec<Bean=<Self::Construct as Construct>::Bean>;
    type Destroy: Destroy<Bean=<Self::Construct as Construct>::Bean>;

    /// Attempts to retrieve a reference to the bean instance, returning an error if the bean is not yet ready.
    fn try_get<'a>() -> crate::Result<&'a <Self::Construct as Construct>::Bean> {
        Self::Spec::holder()
            .get()
            .ok_or(IocError::DependNotReady {
                type_name: Self::Spec::bean_type_name()
            })
    }

    fn get<'a>() -> &'a <Self::Construct as Construct>::Bean {
        Self::try_get().expect("Failed to get bean from context")
    }

    /// Initializes the bean in the context, ensuring it is ready for injection.
    fn init<'a>(ctx: &mut InitCtx) -> crate::Result<&'a <Self::Construct as Construct>::Bean>
    where
        Self::Spec: 'static,
    {
        ctx.get_or_init::<Self>()
    }

    /// Returns a unique identifier for the bean based on its type.
    fn bean_id() -> BeanId
    where
        Self::Spec: 'static,
    {
        Self::Spec::bean_id()
    }

    /// Returns the info for this bean, including its identifier and type name.
    fn info() -> BeanInfo
    where
        Self::Spec: 'static,
    {
        let info = Self::Spec::bean_info();

        trace!("build bean info {} from {} with type {}", info.name, info.spec_name, info.bean_type_name);

        info
    }
}

impl<T> Bean for T
where
    T: Construct<Bean=T> + Destroy<Bean=T> + BeanSpec<Bean=T>,
{
    type Construct = Self;
    type Spec = Self;
    type Destroy = Self;
}

// `DropGuard` is responsible for the cleanup logic of the IoC container.
pub struct DropGuard {
    ready_beans: Vec<(BeanInfo, fn())>,
}

impl Drop for DropGuard {
    /// Automatically performs the cleanup of all registered beans when the `DropGuard` instance is dropped.
    fn drop(&mut self) {
        debug!("Starting cleanup of beans.");
        // Iterate and clean up all beans to ensure resources are properly released.
        for bean_spec in self.ready_beans.iter().rev() {
            debug!("bean {:?} is cleaning", bean_spec);
            // Call the drop function for each bean to perform cleanup.
            (bean_spec.1)();
        }
        // Perform any other necessary cleanup here.
        debug!("Cleanup of beans completed.");
    }
}

impl InitCtx {

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
    pub fn get_or_init<'a, B>(&mut self) -> crate::Result<&'a <B::Spec as BeanSpec>::Bean>
    where
        B: ?Sized + Bean<Spec: 'static>,
    {
        let info = B::info();
        let id = B::bean_id();


        // Check if the bean is already initialized and return it if so.
        if self.ready_bean_ids.contains(&id) {
            return B::try_get();
        }

        // Use the cache to detect potential circular dependencies by checking if the bean
        // is currently in the process of being initialized.
        // Check if the bean is currently being initialized and return an error if so.
        for pending_spec in self.pending_chain.iter() {
            if pending_spec.eq(&id) {
                //todo make it more readable
                return Err(IocError::CircularDependency);
            }
        }
        self.pending_chain.push_back(info);
        debug!("bean {:?} is pending! ", info);

        // The holder's `get_or_try_init` method will attempt to build the bean if it's not already initialized.
        let result = B::Spec::holder()
            .get_or_try_init(||
            <B::Construct as Construct>::build(self)
            );

        let ready_bean = self.pending_chain
            .pop_back()
            .expect("Initialization stack is unexpectedly empty");

        if ready_bean != id {
            panic!("Initialization stack order corrupted");
        }


        if result.is_ok() {
            self.ready_beans.push((ready_bean, || B::Destroy::drop(B::get())));
            self.ready_bean_ids.insert(id);
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

    impl BeanSpec for CfgA {
        type Bean = Self;

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<CfgA> = OnceLock::new();
            &HOLDER
        }
    }

    pub struct A(String);

    impl Construct for A {
        type Bean = Self;

        fn build(ctx: &mut InitCtx) -> crate::Result<Self::Bean> {
            let cfg = ctx.get_or_init::<CfgA>()?;
            Ok(A(cfg.v.clone()))
        }
    }

    impl Destroy for A {
        type Bean = Self;

        fn drop(_: &Self::Bean) {}
    }

    impl BeanSpec for A {
        type Bean = Self;

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<A> = OnceLock::new();
            &HOLDER
        }
    }

    struct B(&'static A, String);

    impl Construct for B {
        type Bean = Self;

        fn build(ctx: &mut InitCtx) -> crate::Result<Self::Bean> {
            let cfg = ctx.get_or_init::<CfgA>()?;
            let a = ctx.get_or_init::<A>()?;
            Ok(B(a, cfg.t.clone()))
        }
    }

    impl Destroy for B {
        type Bean = Self;

        fn drop(_: &Self::Bean) {}
    }

    impl BeanSpec for B {
        type Bean = Self;

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
            bean::{BeanSpec, Construct, Destroy, InitCtx},
            IocError,
        };

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

        impl Construct for A {
            type Bean = Self;

            fn build(_: &mut InitCtx) -> crate::Result<Self::Bean> {
                Ok(A("this is A".to_string()))
            }
        }

        impl BeanSpec for A {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<A> = OnceLock::new();
                &HOLDER
            }
        }

        impl Destroy for A {
            type Bean = Self;
        }


        impl Construct for B {
            type Bean = B;

            fn build(ctx: &mut InitCtx) -> crate::Result<Self::Bean> {
                let a = ctx.get_or_init::<A>()?;
                Ok(B(a, "this is B".to_string()))
            }
        }

        impl BeanSpec for B {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<B> = OnceLock::new();
                &HOLDER
            }
        }

        impl Destroy for B {
            type Bean = Self;
        }

        impl Construct for C {
            type Bean = Self;

            fn build(ctx: &mut InitCtx) -> crate::Result<Self::Bean> {
                let a = ctx.get_or_init::<A>()?;
                let b = ctx.get_or_init::<B>()?;
                Ok(C(a, b, "this is C".to_string()))
            }
        }

        impl BeanSpec for C {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<C> = OnceLock::new();
                &HOLDER
            }
        }

        impl Destroy for C {
            type Bean = Self;
        }

        impl Construct for D {
            type Bean = Self;

            fn build(ctx: &mut InitCtx) -> crate::Result<Self::Bean> {
                let c = ctx.get_or_init::<C>()?;
                let f = ctx.get_or_init::<F>()?;
                Ok(D(c, f, "this is D".to_string()))
            }
        }

        impl BeanSpec for D {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<D> = OnceLock::new();
                &HOLDER
            }
        }

        impl Destroy for D {
            type Bean = Self;
        }

        impl Construct for E {
            type Bean = Self;

            fn build(ctx: &mut InitCtx) -> crate::Result<Self::Bean> {
                let c = ctx.get_or_init::<D>()?;
                Ok(E(c, "this is E".to_string()))
            }
        }

        impl BeanSpec for E {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<E> = OnceLock::new();
                &HOLDER
            }
        }

        impl Destroy for E {
            type Bean = Self;
        }

        impl Construct for F {
            type Bean = Self;

            fn build(ctx: &mut InitCtx) -> crate::Result<Self::Bean> {
                let e = ctx.get_or_init::<E>()?;
                Ok(F(e, "this is E".to_string()))
            }
        }

        impl BeanSpec for F {
            type Bean = Self;

            fn holder<'a>() -> &'a OnceLock<Self::Bean> {
                static HOLDER: OnceLock<F> = OnceLock::new();
                &HOLDER
            }
        }

        impl Destroy for F {
            type Bean = Self;
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
