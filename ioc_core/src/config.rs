use std::fmt::{Debug, Formatter};

use cfg_rs::Configuration;

use crate::IocError;

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

pub struct AppConfigLoader<'a> {
    name: &'a str,
    dir: &'a str,
    profile: &'a str,
}

impl<'a> AppConfigLoader<'a> {
    pub fn new() -> Self {
        Self {
            name: "app",
            dir: ".",
            profile: "prod",
        }
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub fn dir(mut self, dir: &'a str) -> Self {
        self.dir = dir;
        self
    }

    pub fn profile(mut self, profile: &'a str) -> Self {
        self.profile = profile;
        self
    }
}

impl AppConfigLoader<'_> {
    pub fn load(self) -> crate::Result<Config> {
        use cfg_rs::{Configuration, init_cargo_env};
        init_cargo_env!();

        let configuration = Configuration::with_predefined_builder()
            .set_cargo_env(init_cargo_env())
            .set_name(self.name)
            .set_dir(self.dir)
            .set_profile(self.profile)
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

    use crate::{
        BeanSpec,
        init::InitContext,
    };
    use crate::init::InitCtx;

    #[derive(FromConfig)]
    #[config(prefix = "cfg_test")]
    struct Test {
        #[config(name = "hello")]
        v: String,
        //fields...
    }

    impl BeanSpec for Test {
        type Bean = Self;

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<Test> = OnceLock::new();
            &HOLDER
        }

        fn build(ctx: &mut impl InitContext) -> crate::Result<Self::Bean> {
            ctx.get_predefined_config()
        }
    }

    #[test]
    fn it_works() -> Result<(), ConfigError> {
        init_cargo_env!();

        let config = Configuration::with_predefined_builder()
            .set("cfg_test.hello", "world")
            .init()?
            .into();

        let mut ctx = InitCtx::new(config);

        let result = ctx.get_or_init::<Test>()?;

        assert_eq!("world", result.v);

        assert_eq!("world", Test::get().v);
        Ok(())
    }
}