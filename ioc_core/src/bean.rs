use std::{any::{TypeId, type_name}, alloc::Layout};

use log::debug;

pub trait Bean {}

#[derive(Debug)]
pub struct BeanDefinition {
    pub name: &'static str,
    pub type_id: TypeId,
    pub layout: Layout,
}

impl BeanDefinition {
    pub fn of<T: 'static>() -> Self {
        let type_id = TypeId::of::<T>();
        let name = type_name::<T>();
        let layout = Layout::new::<T>();

        debug!("name:{name} id:{type_id:?} layout size:{}", layout.size());

        Self {
            name,
            type_id,
            layout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct A(usize);

    #[test]
    fn it_works() {
        let definition = BeanDefinition::of::<A>();
        assert_eq!(definition.name, "ioc_core::bean::tests::A");
        assert_eq!(definition.type_id, TypeId::of::<A>());
        assert_eq!(definition.layout.size(), (usize::BITS/8) as usize);
    }
}