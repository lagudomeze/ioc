use std::{
    alloc::Layout,
    any::{type_name, TypeId},
};

use log::debug;

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

pub trait BeanTypeHolder {
    type T: Bean + 'static;
}

impl BeanQuery {
    pub fn maybe_none_name<T: 'static>(maybe_name: Option<&'static str>) -> Self {
        if let Some(name) = maybe_name {
            Self::named::<T>(name)
        } else {
            Self::of::<T>()
        }
    }

    pub fn of<T: 'static>() -> Self {
        BeanQuery::OnlyType {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
        }
    }

    pub fn named<T: 'static>(name: &'static str) -> Self {
        BeanQuery::NameAndType {
            name,
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
        }
    }

    pub fn named_from_holder<R: BeanTypeHolder>(name: &'static str) -> Self {
        Self::named::<R::T>(name)
    }

    pub fn from_holder<R: BeanTypeHolder>() -> Self {
        Self::of::<R::T>()
    }
}

pub trait Bean {
    fn dependencies() -> Vec<BeanQuery> {
        Vec::new()
    }

    fn self_qurey() -> BeanQuery
    where
        Self: Sized + 'static,
    {
        BeanQuery::named::<Self>(Self::name())
    }

    fn type_id() -> TypeId
    where
        Self: 'static,
    {
        TypeId::of::<Self>()
    }

    fn name() -> &'static str {
        type_name::<Self>()
    }

    fn type_name() -> &'static str {
        type_name::<Self>()
    }

    fn layout() -> Layout
    where
        Self: Sized,
    {
        Layout::new::<Self>()
    }

    fn definition() -> BeanDefinition
    where
        Self: Sized + 'static,
    {
        let type_id = Self::type_id();
        let name = Self::name();
        let type_name = type_name::<Self>();
        let layout = Self::layout();

        debug!(
            "type name:{name} id:{type_id:?} layout size:{}",
            layout.size()
        );

        BeanDefinition {
            name,
            type_name,
            type_id,
            layout,
            dependencies: Self::dependencies(),
        }
    }
}
#[derive(Debug)]
pub struct BeanDefinition {
    pub name: &'static str,
    pub type_name: &'static str,
    pub type_id: TypeId,
    pub layout: Layout,
    pub dependencies: Vec<BeanQuery>,
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct A(pub usize);

    impl Bean for A {}

    struct B;

    impl Bean for B {
        fn dependencies() -> Vec<BeanQuery> {
            vec![BeanQuery::of::<A>()]
        }
    }

    #[test]
    fn it_works() {
        let definition = A::definition();
        assert_eq!(definition.name, "ioc_core::bean::tests::A");
        assert_eq!(definition.type_id, TypeId::of::<A>());
        assert_eq!(definition.layout, Layout::new::<usize>());
    }
}
