//! ## ioc - A simple dependency injection library for Rust
//! `ioc` is a dependency injection library that provides a simple way to manage and resolve dependencies.
//! ## The [`run!`](run) macro
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
//! ```rust
//! use ioc::run;
//!
//! fn main() -> ioc::Result<()>{
//!
//!     // run!(); // Uses default values for name, dir, and profile
//!     run!(name = "my_app", dir = ".", profile = "prod");
//!     Ok(())
//! }
//! ```
//!
//! ## The [`Bean`](ioc_derive::Bean) derive macro
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


#[doc(hidden)]
pub use linkme::{self, *};

pub use ioc_core::{
    AppConfigLoader,
    Bean,
    BeanFactory,
    Config,
    Context,
    IocError,
    Result,
    MethodType,
    BeanFamily,
    Init,
    Wrapper
};
use ioc_core::DropGuard;
pub use ioc_derive::{Bean, preload_mods, load_config, load_types};
pub use log::{log_init, LogPatcher};

pub mod log;

/// See module level documentation for more information.
#[macro_export]
macro_rules! run {
    ($($field:ident = $value:expr),* $(,)?) => {
        use ioc::{load_config, log_init, preload_mods, run_app, Context};

        log_init()?;

        let loader = load_config!($($field: $value,)*);

        let  config = loader.load()?;

        let mut ctx = Context::new(config);

        all_types_with::<Init>(&mut ctx)?;

        ctx.complete()
    }
}
/// This is a global-distributed slice used to collect all bean factory functions.
#[doc(hidden)]
#[distributed_slice]
pub static BEAN_COLLECTOR: [fn(&mut Context) -> Result<()>] = [..];

/// Runs the application, initializes all beans, and completes dependency injection.
///
/// # Parameters
///
/// * `loader` - A configuration [loader](AppConfigLoader) to load [configuration](Config).
///
/// # Returns
///
/// If the application runs successfully, it returns `Ok(())`. If an error occurs during the run, it returns `Err(IocError)`.
pub fn run_app(loader: AppConfigLoader) -> Result<DropGuard> {

    let  config = loader.load()?;
    let mut ctx = Context::new(config);
    for collect in BEAN_COLLECTOR {
        collect(&mut ctx)?;
    }

    Ok(ctx.complete())
}