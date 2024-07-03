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
//!     run!(name = "my_app", dir = ".", profile = "prod");
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
//! use ioc::{Bean, BeanFactory, Context};
//!
//! #[derive(Bean)]
//! struct A;
//!
//! #[derive(Bean)]
//! #[custom_factory]
//! struct AnotherBeanA;
//!
//! impl BeanFactory for AnotherBeanA {
//!     type Bean = A;
//!
//!     fn build(ctx: &mut Context) -> ioc::Result<Self::Bean> {
//!         Ok(A)
//!     }
//! }
//!
//! #[derive(Bean)]
//! #[name("my_bean")]
//! pub struct MyBean {
//!     #[inject]
//!     a: &'static A,
//!     #[inject(AnotherBeanA)]
//!     another_a: &'static A,
//!     #[value("config.key")]
//!     config_value: String,
//! }
//! ```


pub use ioc_core::{
    AppConfigLoader,
    Bean,
    BeanFactory,
    BeanFamily,
    Config,
    Context,
    Init,
    IocError,
    MethodType,
    Result,
    Wrapper
};
pub use ioc_core_derive::{Bean, load_config};
pub use ioc_macro::{export, import};
#[cfg(feature = "mvc")]
pub use ioc_mvc::{mvc, run_mvc, OpenApi};
pub use log::{log_init, LogPatcher};

pub mod log;

pub fn pre_init(_ctx: &mut Context) -> Result<()> {
    #[cfg(feature = "mvc")]
    _ctx.get_or_init::<ioc_mvc::WebConfig>()?;
    Ok(())
}

/// See module level documentation for more information.
#[macro_export]
macro_rules! run {
    ($($field:ident = $value:expr),* $(,)?) => {
        {
            use ioc::*;

            log_init()?;

            let loader = load_config!($($field: $value,)*);

            let config = loader.load()?;

            let mut ctx = Context::new(config);

            pre_init(&mut ctx)?;

            import!();

            ctx.complete()
        }
    }
}