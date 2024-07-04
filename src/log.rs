#[cfg(feature = "env_logger")]
pub use env_logger::*;
pub use log::*;

#[cfg(not(any(feature = "env_logger", feature = "tracing_log")))]
pub use none::*;
#[cfg(feature = "tracing_log")]
pub use tracing_log::*;

#[cfg(all(feature = "env_logger", feature = "tracing_log"))]
compile_error!("feature \"env_logger\" and feature \"tracing_log\" cannot be enabled at the same time");

#[cfg(feature = "env_logger")]
mod env_logger {
    use env_logger::{builder, Env};
    use tracing::level_filters::LevelFilter;
    use ioc_core::{Bean, BeanFactory};

    pub struct LogOptions<'a> {
        env: Env<'a>,
    }

    impl<'a> LogOptions<'a> {
        pub fn new() -> Self {
            Self {
                env: Default::default()
            }
        }

        pub fn debug(mut self, debug : bool) -> Self {
            if debug {
                self.env = Env::from("RUST_LOG=debug");
            }
            self
        }

        pub fn init(self) -> crate::Result<()> {
            builder().env(self.env)
                .try_init()?;
            Ok(())
        }
    }
}

#[cfg(feature = "tracing_log")]
mod tracing_log {
    use std::sync::OnceLock;

    use tracing_subscriber::{
        EnvFilter,
        filter::Directive,
        filter::LevelFilter,
        fmt::Formatter,
        reload::Handle
    };

    use ioc_core::{Bean, BeanFactory, Context};

    pub struct LogOptions {
        default_directive: Directive,
    }

    impl LogOptions {
        pub fn new() -> Self {
            Self {
                default_directive: LevelFilter::INFO.into()
            }
        }

        pub fn debug(mut self, debug : bool) -> Self {
            if debug {
                self.default_directive = LevelFilter::DEBUG.into();
            }
            self
        }

        pub fn init(self) -> crate::Result<()> {
            let filter = EnvFilter::builder()
                .with_default_directive(self.default_directive)
                .from_env_lossy();

            let builder = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_filter_reloading();

            let handle = builder.reload_handle();

            builder.init();

            LogPatcher::holder().get_or_init(|| { LogPatcher(handle) });

            Ok(())
        }
    }

    pub struct LogPatcher(Handle<EnvFilter, Formatter>);

    impl LogPatcher {
        pub fn reload<'a, I>(&self, value: I) -> crate::Result<()>
        where
            I: IntoIterator<Item: AsRef<str>>,
        {
            let mut env_filter = EnvFilter::from_default_env();
            for i in value {
                let directive = i.as_ref().parse().map_err(anyhow::Error::new)?;
                env_filter = env_filter.add_directive(directive)
            }

            self.0.modify(|filter: &mut EnvFilter| {
                *filter = env_filter;
            }).map_err(anyhow::Error::new)?;
            Ok(())
        }

        pub fn to_string(&self) -> crate::Result<String> {
            let result = self.0.with_current(|filter: &EnvFilter| {
                filter.to_string()
            }).map_err(anyhow::Error::new)?;
            Ok(result)
        }
    }

    impl BeanFactory for LogPatcher {
        type Bean = Self;

        fn build(_ctx: &mut Context) -> ioc_core::Result<Self::Bean> {
            panic!("do not run here!")
        }
    }

    impl Bean for LogPatcher {
        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<LogPatcher> = OnceLock::new();
            &HOLDER
        }
    }
}

#[cfg(not(any(feature = "env_logger", feature = "tracing_log")))]
mod none {
    pub struct LogOptions;

    impl LogOptions {
        pub fn new() -> Self {
            Self
        }

        pub fn debug(mut self, _ : bool) -> Self {
            self
        }

        pub fn init(self) -> crate::Result<()> {
            println!("no env_logger and tracing_log use your log implement!");
            Ok(())
        }
    }
}