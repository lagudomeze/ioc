use std::any::{type_name, TypeId};
use std::sync::OnceLock;

use log::{debug, info, warn};

use crate::config::IocConfig;

pub trait Bean {
    fn dependencies() -> Vec<BeanQuery> {
        Vec::new()
    }

    fn name() -> &'static str {
        type_name::<Self>()
    }
}

#[derive(Debug)]
pub struct BeanDefinition {
    pub name: &'static str,
    pub type_name: &'static str,
    pub type_id: TypeId,
    pub dependencies: Vec<BeanQuery>,
}

pub fn definition<B>() -> BeanDefinition where B: 'static + Bean {
    let name = B::name();
    let dependencies = B::dependencies();

    let type_id = TypeId::of::<B>();
    let type_name = type_name::<B>();

    debug!(" name:{name} type:{type_name} id:{type_id:?}");

    BeanDefinition {
        name,
        type_name,
        type_id,
        dependencies,
    }
}

#[derive(Debug)]
pub enum BeanQuery {
    OnlyType {
        type_id: TypeId,
        type_name: &'static str,
    },
    NameAndType {
        name: &'static str,
        type_id: TypeId,
        type_name: &'static str,
    },
}

pub fn of<T: 'static>() -> BeanQuery {
    BeanQuery::OnlyType {
        type_id: TypeId::of::<T>(),
        type_name: type_name::<T>(),
    }
}

pub fn named<T: 'static>(name: &'static str) -> BeanQuery {
    BeanQuery::NameAndType {
        name,
        type_id: TypeId::of::<T>(),
        type_name: type_name::<T>(),
    }
}

pub trait Factory {
    type Config;

    type Product: Sized;

    fn build(config: &Self::Config) -> crate::Result<Self::Product>;

    fn destroy(_product: &Self::Product) {}
}

pub trait Singleton: Factory {
    fn holder<'a>() -> &'a OnceLock<Self::Product>;

    fn get<'a>() -> &'a Self::Product {
        Self::holder().get().expect("")
    }

    fn init<'a>(config: &Self::Config) -> crate::Result<&'a Self::Product> {
        Ok(Self::holder().get_or_try_init(|| Self::build(config))?)
    }

    fn drop() {
        if let Some(s) = Self::holder().get() {
            Self::destroy(s);
            info!("{} is dropped!", type_name::<Self::Product>());
        } else {
            warn!("{} is not init! so skip destroy", type_name::<Self::Product>());
        }
    }
}

pub trait BeanSingleton: Singleton {
}

impl<B> BeanSingleton for B where
    B: Singleton,
    Self::Product: Bean,
    Self::Config: IocConfig {}

#[cfg(test)]
mod tests {
    use cfg_rs::*;

    use super::*;

    #[derive(FromConfig)]
    #[config(prefix = "cfg_a")]
    pub struct CfgA {
        #[config(name = "a")]
        v: String,
        #[config(name = "b")]
        t: String,
    }

    impl Singleton for CfgA {
        fn holder<'a>() -> &'a OnceLock<Self::Product> {
            static HOLDER: OnceLock<CfgA> = OnceLock::new();
            &HOLDER
        }
    }

    pub struct A(String);

    impl Bean for A {}

    impl Factory for A {
        type Config = CfgA;
        type Product = A;

        fn build(config: &Self::Config) -> crate::Result<Self::Product> {
            Ok(A(config.v.clone()))
        }
    }

    impl Singleton for A {
        fn holder<'a>() -> &'a OnceLock<Self> {
            static HOLDER: OnceLock<A> = OnceLock::new();
            &HOLDER
        }
    }


    struct B(&'static A, String);

    impl Bean for B {
        fn dependencies() -> Vec<BeanQuery> {
            vec![of::<A>()]
        }
    }

    impl Factory for B {
        type Config = CfgA;
        type Product = B;

        fn build(config: &Self::Config) -> crate::Result<Self::Product> {
            Ok(B(A::get(), config.t.clone()))
        }
    }

    impl Singleton for B {
        fn holder<'a>() -> &'a OnceLock<Self> {
            static HOLDER: OnceLock<B> = OnceLock::new();
            &HOLDER
        }
    }


    #[test]
    fn test_inject_init() {
        init_cargo_env!();
        let config = Configuration::with_predefined_builder()
            .set("cfg_a.a", "data")
            .set("cfg_a.b", "babel")
            .init().unwrap();

        let init_a : fn(&Configuration) -> crate::Result<()> = |config: &Configuration| {
            let cfg = CfgA::init(&config)?;
            A::init(&cfg)?;
            Ok(())
        };

        let drop_a : fn() = A::drop;

        let init_b : fn(&Configuration) -> crate::Result<()> = |config: &Configuration| {
            let cfg = CfgA::init(&config)?;
            B::init(&cfg)?;
            Ok(())
        };

        let drop_b : fn() = B::drop;

        init_a(&config).unwrap();
        let a = A::get();
        assert_eq!("data", a.0);

        init_b(&config).unwrap();
        let b = B::get();
        assert_eq!("babel", b.1);
        assert_eq!(a as *const A, b.0 as *const A);

        assert_eq!(a as *const A, A::get() as *const A);
        assert_eq!(b as *const B, B::get() as *const B);

        drop_b();
        drop_a();
    }

    #[test]
    fn it_works() {
        let definition = definition::<A>();
        assert_eq!(definition.name, "ioc_core::bean::tests::A");
        assert_eq!(definition.type_id, TypeId::of::<A>());
    }
}
