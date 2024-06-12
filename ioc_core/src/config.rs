use cfg_rs::{Configuration, FromConfigWithPrefix};

use crate::{Bean, BeanHolder};
use crate::bean::{BeanFactory, Context};

impl<C> Bean for C where C: FromConfigWithPrefix {}

impl<C> BeanFactory for C where C: FromConfigWithPrefix {
    type Bean = Self;

    fn build(self, ctx: &mut Context) -> crate::Result<Self::Bean> {
        let config = ctx.init::<Configuration>()?;
        Ok(config.get_predefined()?)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use cfg_rs::*;

    use crate::bean::{BeanFactory, Context};
    use crate::BeanHolder;

    struct ConfigTest;

    impl BeanFactory for ConfigTest {
        type Bean = Configuration;

        fn build(self, _: &mut Context) -> crate::Result<Self::Bean> {
            init_cargo_env!();

            let config = Configuration::with_predefined_builder()
                .set("cfg_test.hello", "world")
                .init()?;

            Ok(config)
        }
    }

    impl BeanHolder for Configuration {
        type Factory = ConfigTest;

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<Configuration> = OnceLock::new();
            &HOLDER
        }
    }

    #[derive(FromConfig)]
    #[config(prefix = "cfg_test")]
    struct Test {
        #[config(name = "hello")]
        v: String,
        //fields...
    }

    impl BeanHolder for Test {
        type Factory = Test;

        fn holder<'a>() -> &'a OnceLock<Self::Bean> {
            static HOLDER: OnceLock<Test> = OnceLock::new();
            &HOLDER
        }
    }

    #[test]
    fn it_works() -> Result<(), ConfigError> {
        let mut ctx = Context::new();

        let result = ctx.init::<Test>()?;

        assert_eq!("world", result.v);

        assert_eq!("world", Test::get().v);
        Ok(())
    }
}