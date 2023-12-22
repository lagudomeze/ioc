use std::{
    alloc::Layout,
    any::{type_name, TypeId},
    mem::needs_drop,
    ptr::drop_in_place,
};

use log::debug;

pub trait Bean {
    fn dependencies() -> Vec<Dependency> {
        Vec::new()
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

    fn layout() -> Layout
    where
        Self: Sized,
    {
        Layout::new::<Self>()
    }

    fn maybe_drop() -> Option<DropMethod>
    where
        Self: Sized,
    {
        if needs_drop::<Self>() {
            Some(drop::<Self>)
        } else {
            None
        }
    }

    fn definition() -> BeanDefinition
    where
        Self: Sized + 'static,
    {
        let type_id = Self::type_id();
        let name = Self::name();
        let type_name = type_name::<Self>();
        let layout = Self::layout();
        let maybe_drop = Self::maybe_drop();

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
            maybe_drop,
        }
    }
}


pub type DropMethod = unsafe fn(*mut u8);

unsafe fn drop<T>(ptr: *mut u8) {
    drop_in_place(ptr.cast::<T>());
}

#[derive(Debug)]
pub struct Dependency {
    pub name: Option<&'static str>,
    pub type_id: TypeId,
    pub type_name: &'static str,
}

impl Dependency {
    pub fn of<T>() -> Dependency
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let type_name = type_name::<T>();
        Dependency {
            name: None,
            type_name,
            type_id,
        }
    }
    pub fn with_name<T>(name: &'static str) -> Dependency
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let type_name = type_name::<T>();
        Dependency {
            name: Some(name),
            type_name,
            type_id,
        }
    }
}

#[derive(Debug)]
pub struct BeanDefinition {
    pub name: &'static str,
    pub type_name: &'static str,
    pub type_id: TypeId,
    pub layout: Layout,
    pub dependencies: Vec<Dependency>,
    pub maybe_drop: Option<DropMethod>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct A(usize);

    impl Bean for A {
    }

    struct B;

    impl Bean for B {
        fn dependencies() -> Vec<Dependency> {
            vec![Dependency::of::<A>()]
        }
    }

    #[test]
    fn it_works() {
        let definition = A::definition();
        assert_eq!(definition.name, "ioc_core::bean::tests::A");
        assert_eq!(definition.type_id, TypeId::of::<A>());
        assert_eq!(definition.layout.size(), (usize::BITS / 8) as usize);
    }
}
