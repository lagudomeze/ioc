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
//!     fn build(ctx: &mut Context) -> ioc_core::Result<Self::Bean> {
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
pub use linkme;
use linkme::distributed_slice;

/// `ioc` is a dependency injection library that provides a simple way to manage and resolve dependencies.
pub use ioc_core::{
    AppConfigLoader,
    AppName,
    Bean,
    BeanFactory,
    Config,
    ConfigPath,
    ConfigProfile,
    Context,
    IocError,
    Result,
};
pub use ioc_derive::{Bean, preload_mods};

/// See module level documentation for more information.
#[macro_export]
macro_rules! run {
    ($($field:ident = $value:expr),* $(,)?) => {
        use ioc::{preload_mods, AppConfigLoader, run_app};
        preload_mods!();
        let config = AppConfigLoader {
            $(
                $field: $value.into(),
            )*
            ..Default::default()
        }.load()?;
        run_app(config)?;
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
/// * `config` - A configuration object containing the application's configuration information.
///
/// # Returns
///
/// If the application runs successfully, it returns `Ok(())`. If an error occurs during the run, it returns `Err(IocError)`.
pub fn run_app(config: Config) -> Result<()> {
    env_logger::init();

    let mut ctx = Context::new(config);
    for collect in BEAN_COLLECTOR {
        collect(&mut ctx)?;
    }
    let _ = ctx.complete();

    Ok(())
}