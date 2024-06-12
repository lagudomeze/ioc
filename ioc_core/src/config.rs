use cfg_rs::{Configuration, FromConfigWithPrefix};

use crate::Bean;
use crate::bean::{BeanFactory, Context};

impl<C> BeanFactory for C where C: FromConfigWithPrefix + Bean {
    type Bean = Self;

    fn build(ctx: &mut Context) -> crate::Result<Self::Bean> {
        Ok(ctx.config.source.get_predefined()?)
    }
}

pub struct Config {
    source: Configuration,
}

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

    use crate::{Bean, BeanHolder};
    use crate::bean::Context;

    #[derive(FromConfig)]
    #[config(prefix = "cfg_test")]
    struct Test {
        #[config(name = "hello")]
        v: String,
        //fields...
    }

    impl Bean for Test {}

    impl BeanHolder for Test {
        type Bean = Test;
        type Factory = Test;

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