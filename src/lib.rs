//! ## ioc - A simple dependency injection library for Rust
//! `ioc` is a dependency injection library that provides a simple way to manage and resolve dependencies.
//! ## The [`run!`](run) derive
//!
//! Used to run the application, initialize all beans, and complete dependency injection.
//!
//! ### Parameters
//!
//! * `name` - The name of the application. Default is the current module's name (`env!("CARGO_PKG_NAME")`).
//! * `dir` - The path to the configuration file. Default is the current path (`"."`).
//! * `profile` - The profile of the configuration file (prod/dev). Default is `"prod"`.
//!
//! ### Example
//!
//! ```no_run
//! use ioc::run;
//!
//! // this is generated by the macro `export` from `ioc_macro` module
//! pub fn all_beans_with<F: ioc::BeanFamily>(ctx: F::Ctx) -> ioc::Result<F::Ctx> {
//!     Ok(ctx)
//! }
//! pub fn all_mvcs<T>(api: T) -> impl poem_openapi::OpenApi
//! where
//!     T: poem_openapi::OpenApi,
//! {
//!     api
//! }
//!
//! fn main() -> ioc::Result<()>{
//!
//!     // run!(); // Uses default values for name, dir, and profile
//!     run!(
//!         name = "my_app";
//!         dir = ".";
//!         profile = "prod";
//!     );
//!     Ok(())
//! }
//! ```
//!
//! ## The [`Bean`](ioc_derive::Bean) derive derive
//!
//! Used to define a [bean](ioc_core::Bean), which can automatically implement the [`BeanFactory`] and `Bean` traits.
//!
//! ### Attributes
//!
//! * `inject` - Used to inject other beans. If the type is not specified, the field's type will be used.
//! * `value` - Used to get a value from the configuration.
//! * `name` - Used to specify the name of the bean. If not specified, the struct's name will be used.
//! * `custom_factory` - Used to specify a custom factory method. If this attribute is specified, a factory method will not be automatically generated.
//!
//! ### Example
//!
//! ```rust
//! use ioc::*;
//!
//! #[derive(Bean)]
//! #[bean(ioc_crate = ioc)]
//! struct A;
//!
//! struct AnotherBeanA;
//!
//! #[bean(ioc_crate = ioc, name = "__a__")]
//! impl BeanSpec for AnotherBeanA {
//!     type Bean = A;
//!
//!     fn build<I>(ctx: &mut I) -> Result<Self::Bean> where I: InitContext {
//!         A::build(ctx)
//!     }
//! }
//!
//!
//! #[derive(Bean)]
//! #[bean(name = "my_bean", ioc_crate = ioc)]
//! pub struct MyBean {
//!     #[inject(bean)]
//!     a: &'static A,
//!     #[inject(bean_with = AnotherBeanA)]
//!     another_a: &'static A,
//!     #[inject(config = "config.key")]
//!     config_value: String,
//! }
//! ```


pub use ioc_core::{
    AppConfigLoader,
    BeanFamily,
    BeanSpec,
    Config,
    Init,
    InitContext,
    InitCtx,
    IocError,
    Method,
    Result,
    Wrapper
};
pub use ioc_core_derive::{Bean, bean};
pub use ioc_macro::{export, import};
#[cfg(feature = "mvc")]
pub use ioc_mvc::{mvc, OpenApi, OpenApiExt, run_mvc, WebConfig};

pub mod log;

pub fn all_beans_with<F: BeanFamily>(ctx: F::Ctx) -> Result<F::Ctx> {
    #[cfg(feature = "mvc")]
    let ctx = {
        use ioc_core::Method;
        F::Method::<WebConfig>::run(ctx)?
    };
    Ok(ctx)
}

#[cfg(feature = "mvc")]
pub fn all_mvcs<T>(api: T) -> impl OpenApiExt
where
    T: OpenApiExt,
{
    api
}

#[doc(hidden)]
pub mod __private {
    pub use ioc_core::{
        AppConfigLoader,
        InitCtx,
        Result,
    };

    pub use crate::log::LogOptions;
}

#[macro_export]
macro_rules! init_logger {
    () => {
        use ioc::__private;

        __private::LogOptions::new().init()?;
    };
    (debug = $debug:expr) => {
        use ioc::__private;

        __private::LogOptions::new().debug($debug).init()?;
    };
}

#[macro_export]
macro_rules! init_context {
    (
        $(name = $name:expr;)?
        $(dir = $dir:expr;)?
        $(profile = $profile:expr;)?
    ) => {
        {
            use ioc::__private;

            let mut name = env!("CARGO_PKG_NAME");
            $(name = $name;)?

            let config = __private::AppConfigLoader::new()
                .name(name)
                $(.dir($dir))?
                $(.profile($profile))?
                .load()?;

            __private::InitCtx::new(config)
        }
    };
}

#[macro_export]
macro_rules! run {
    (
        // log
        $(debug = $debug:expr;)?

        // config
        $(name = $name:expr;)?
        $(dir = $dir:expr;)?
        $(profile = $profile:expr;)?

        // import crates
        $(use_crate = $use_crate:expr;)?
        $(crates($($dep_crate:path),*);)?

    ) => {
        {
            // init logger
            $crate::init_logger!($(debug = $debug)?);

            // init context
            let mut ctx = $crate::init_context!(
                $(name = $name;)?
                $(dir = $dir;)?
                $(profile = $profile;)?
            );

            // import and run mvc(maybe)
            $crate::import!(
                $(use_crate = $use_crate,)?
                $(crates(ioc,$($dep_crate),*))?
            );

            ctx.complete()
        }
    }
}