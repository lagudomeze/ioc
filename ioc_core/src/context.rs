use std::{any::type_name, ops::Deref};
use crate::bean::{Bean, Dependency};

#[derive(Debug)]
pub struct Ref<T> {
    ref_ptr: *const T,
    self_ptr: *const Self,
    marker: std::marker::PhantomData<T>,
}

impl<T> std::fmt::Display for Ref<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = type_name::<T>();
        write!(f, "Bean Ref of {type_name}, self ptr {:p}, target ptr {:p}", self.self_ptr, self.ref_ptr)
    }
}

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = self as *const Self;
        assert_eq!(ptr, self.self_ptr);

        let target_addr = ptr
            .addr()
            .wrapping_add_signed(self.inner_offset)
            .wrapping_add_signed(self.parent_offset);
        let target_ptr = from_exposed_addr::<Self::Target>(target_addr);

        unsafe { target_ptr.as_ref().unwrap() }
    }
}


pub trait AppContext {
    
}

pub trait BeanFactory {
    type Type : Bean; 
}