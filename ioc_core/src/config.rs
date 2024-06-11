use cfg_rs::{Configuration, FromConfigWithPrefix};

use crate::Factory;

pub trait IocConfig: FromConfigWithPrefix + Factory<Config=Configuration, Product=Self> {}

impl<C> Factory for C where C: FromConfigWithPrefix {
    type Config = Configuration;
    type Product = Self;

    fn build(config: &Self::Config) -> crate::Result<Self> {
        Ok(config.get_predefined()?)
    }
}

impl<C> IocConfig for C where C: FromConfigWithPrefix {}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use cfg_rs::*;

    use crate::bean::Singleton;

    #[derive(FromConfig)]
    #[config(prefix = "cfg_test")]
    struct Test {
        #[config(name = "hello")]
        v: String,
        //fields...
    }

    impl Singleton for Test {
        fn holder<'a>() -> &'a OnceLock<Self> {
            static HOLDER: OnceLock<Test> = OnceLock::new();
            &HOLDER
        }
    }

    #[test]
    fn it_works() -> Result<(), ConfigError> {
        init_cargo_env!();
        let config = Configuration::with_predefined_builder()
            .set("cfg_test.hello", "world")
            .init()?;

        let result = Test::init(&config).unwrap();
        assert_eq!("world", result.v);

        assert_eq!("world", Test::get().v);
        Ok(())
    }
}