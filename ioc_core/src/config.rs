use std::fmt::{Debug, Formatter};

use cfg_rs::{Configuration, FromConfigWithPrefix};

use crate::bean::{BeanFactory, Context};
use crate::IocError;

/// BeanFactory for Configuration which implements `cfg_rs::FromConfigWithPrefix`
impl<C> BeanFactory for C
where
    C: FromConfigWithPrefix,
{
    type Bean = Self;

    fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
        Ok(ctx.config.source.get_predefined()?)
    }
}

/// Ioc Context Configuration, just simply wrap `cfg_rs::Configuration`
pub struct Config {
    /// source of configuration
    pub(crate) source: Configuration,
}

impl Debug for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("source", &"Configuration")
            .finish()
    }
}

pub struct AppName(String);

impl Default for AppName {
    fn default() -> Self {
        let string = std::env::var("CARGO_PKG_NAME")
            .unwrap_or("app".to_string());
        Self(string)
    }
}

impl<T> From<T> for AppName
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

pub struct ConfigPath(String);

impl Default for ConfigPath {
    fn default() -> Self {
        Self(".".to_string())
    }
}

impl<T> From<T> for ConfigPath
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}


pub struct ConfigProfile(String);

impl Default for ConfigProfile {
    fn default() -> Self {
        Self("prod".to_string())
    }
}

impl<T> From<T> for ConfigProfile
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

pub struct AppConfigLoader {
    pub name: AppName,
    pub dir: ConfigPath,
    pub profile: ConfigProfile,
}

impl Default for AppConfigLoader {
    fn default() -> Self {
        Self {
            name: Default::default(),
            dir: Default::default(),
            profile: Default::default(),
        }
    }
}

impl AppConfigLoader {
    pub fn load(self) -> crate::Result<Config> {
        use cfg_rs::{Configuration, init_cargo_env};
        init_cargo_env!();

        let configuration = Configuration::with_predefined_builder()
            .set_cargo_env(init_cargo_env())
            .set_name(self.name.0)
            .set_dir(self.dir.0)
            .set_profile(self.profile.0)
            .init()
            .map_err(IocError::from)?;

        Ok(Config::from(configuration))
    }
}

/// Convert Configuration to Config
impl From<Configuration> for Config {
    fn from(source: Configuration) -> Self {
        Self {
            source
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use cfg_rs::*;

    use crate::Bean;
    use crate::bean::Context;

    #[derive(FromConfig)]
    #[config(prefix = "cfg_test")]
    struct Test {
        #[config(name = "hello")]
        v: String,
        //fields...
    }

    impl Bean for Test {
        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<Test> = OnceLock::new();
            &HOLDER
        }
    }

    #[test]
    fn it_works() -> Result<(), ConfigError> {
        init_cargo_env!();

        let config = Configuration::with_predefined_builder()
            .set("cfg_test.hello", "world")
            .init()?
            .into();

        let mut ctx = Context::new(config);

        let result = ctx.get_or_init::<Test>()?;

        assert_eq!("world", result.v);

        assert_eq!("world", Test::get().v);
        Ok(())
    }
}