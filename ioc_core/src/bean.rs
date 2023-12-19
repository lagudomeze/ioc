use std::{
    alloc::Layout,
    any::{type_name, TypeId},
    mem::needs_drop,
    ptr::drop_in_place,
};

use log::debug;

use crate::error::IocError;

pub trait Bean {
    fn definition() -> BeanDefinition
    where
        Self: Sized + 'static,
    {
        let type_id = TypeId::of::<Self>();
        let name = type_name::<Self>();
        let layout = Layout::new::<Self>();

        let maybe_drop: Option<DropMethod> = if needs_drop::<Self>() {
            Some(drop::<Self>)
        } else {
            None
        };

        debug!("name:{name} id:{type_id:?} layout size:{}", layout.size());

        BeanDefinition {
            name,
            type_id,
            layout,
            maybe_drop,
        }
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct BeanId(usize);

pub trait BeanContainer {
    fn id<T: Bean>() -> Result<BeanId, IocError> {
        Err(IocError::NotRegisteredBean {
            type_name: type_name::<T>(),
        })
    }
}

pub type DropMethod = unsafe fn(*mut u8);

unsafe fn drop<T>(ptr: *mut u8) {
    drop_in_place(ptr.cast::<T>());
}

#[derive(Debug)]
pub struct BeanDefinition {
    pub name: &'static str,
    pub type_id: TypeId,
    pub layout: Layout,
    pub maybe_drop: Option<DropMethod>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct A(usize);

    impl Bean for A {}

    #[test]
    fn it_works() {
        let definition = A::definition();
        assert_eq!(definition.name, "ioc_core::bean::tests::A");
        assert_eq!(definition.type_id, TypeId::of::<A>());
        assert_eq!(definition.layout.size(), (usize::BITS / 8) as usize);
    }
}
