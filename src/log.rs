use std::sync::OnceLock;

#[cfg(feature = "env_logger")]
pub use env_logger::*;
pub use log::*;

use ioc_core::{Bean, BeanFactory, Context};
#[cfg(not(any(feature = "env_logger", feature = "tracing_log")))]
pub use none::*;
#[cfg(feature = "tracing_log")]
pub use tracing_log::*;

#[cfg(all(feature = "env_logger", feature = "tracing_log"))]
compile_error!("feature \"env_logger\" and feature \"tracing_log\" cannot be enabled at the same time");

#[cfg(feature = "env_logger")]
mod env_logger {
    use std::sync::OnceLock;

    use ioc_core::{Bean, BeanFactory, Context};

    #[cfg(feature = "env_logger")]
    pub fn log_init() -> crate::Result<()> {
        env_logger::init();
        Ok(())
    }

    pub struct LogPatcher;
}

#[cfg(feature = "tracing_log")]
mod tracing_log {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{
        EnvFilter,
        fmt::Formatter,
        reload::Handle,
    };

    use ioc_core::Bean;

    pub fn log_init() -> crate::Result<()> {
        let filter = EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy();

        let builder = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_filter_reloading();

        let handle = builder.reload_handle();

        builder.init();

        LogPatcher::holder().get_or_init(|| { LogPatcher(handle) });

        Ok(())
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
}

#[cfg(not(any(feature = "env_logger", feature = "tracing_log")))]
mod none {
    pub fn log_init() -> crate::Result<()> {
        //do nothing
        Ok(())
    }

    pub struct LogPatcher;
}

impl BeanFactory for LogPatcher {
    type Bean = Self;

    fn build(_ctx: &mut Context) -> ioc_core::Result<Self::Bean> {
        panic!("do not run here!")
    }
}

impl Bean for LogPatcher {
    fn holder<'a>() -> &'a OnceLock<Self::Bean> {
        static HOLDER: OnceLock<crate::LogPatcher> = OnceLock::new();
        &HOLDER
    }
}