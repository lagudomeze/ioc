use std::{alloc::Layout, any::TypeId, collections::HashMap, ptr::drop_in_place};

use crate::bean::{Bean, BeanDefinition};

pub type BeanId = usize;

pub struct BeanContainer {
    ptr: Box<[u8]>,
    bean_offsets: Box<[usize]>,
    bean_definitions: Box<[BeanDefinition]>,
    type_id_bean_id_map: HashMap<TypeId, BeanId>,
    name_bean_ids_map: HashMap<&'static str, Vec<BeanId>>,
    drop_methods: HashMap<BeanId, unsafe fn(ptr: &mut [u8], offset: usize)>,
    layout: Layout,
}


fn mut_ptr<T>(ptr: &mut [u8], offset: usize) -> *mut T {
    let ptr = ptr.as_mut_ptr();
    let ptr = ptr.wrapping_add(offset);
    ptr.cast::<T>()
}

fn ptr<T>(ptr: &[u8], offset: usize) -> *const T {
    let ptr = ptr.as_ptr();
    let ptr = ptr.wrapping_add(offset);
    ptr.cast::<T>()
}

impl BeanContainer {
    pub fn find_ref_by_id<T: Bean>(&self, id: BeanId) -> Option<&T> {
        let offset = *self.bean_offsets.get(id)?;
        let ptr = ptr::<T>(&self.ptr, offset);
        unsafe { ptr.as_ref() }
    }

    pub fn find_ref<T: 'static + Bean>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let id = *self.type_id_bean_id_map.get(&type_id)?;
        self.find_ref_by_id(id)
    }

    fn find_mut_ref_by_id<T: Bean>(&mut self, id: BeanId) -> Option<&mut T> {
        let offset = *self.bean_offsets.get(id)?;
        let ptr = mut_ptr::<T>(&mut self.ptr, offset);
        unsafe { ptr.as_mut() }
    }

    fn find_mut_ref<T: 'static + Bean>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let id = *self.type_id_bean_id_map.get(&type_id)?;
        self.find_mut_ref_by_id(id)
    }
}

impl Drop for BeanContainer {
    fn drop(&mut self) {
        for (id, drop_method) in self.drop_methods.iter() {
            if let Some(offset) = self.bean_offsets.get(*id) {
                unsafe {
                    drop_method(&mut self.ptr, *offset);
                }
            }
        }
    }
}

pub struct BeanContainerBuilder {
    bean_definitions: Vec<BeanDefinition>,
    drop_methods: HashMap<BeanId, unsafe fn(ptr: &mut [u8], offset: usize)>,
}

unsafe fn drop<T>(ptr: &mut [u8], offset: usize) {
    drop_in_place(mut_ptr::<T>(ptr, offset));
}

impl BeanContainerBuilder {
    pub fn push<T: 'static + Bean>(&mut self) {
        let id = self.bean_definitions.len();
        self.bean_definitions.push(T::definition());
        if std::mem::needs_drop::<T>() {
            self.drop_methods.entry(id).or_insert(drop::<T>);
        }
    }

    pub fn build(self) -> BeanContainer {
        let BeanContainerBuilder {
            bean_definitions,
            drop_methods,
        } = self;

        let mut layout = Layout::from_size_align(0, 8).unwrap();
        let mut bean_offsets = vec![];

        let mut type_id_bean_id_map = HashMap::with_capacity(16);
        let mut name_bean_ids_map = HashMap::with_capacity(16);

        let mut next_id = 0;

        for definition in bean_definitions.iter() {
            let (new_layout, offset) = layout.extend(definition.layout).unwrap();
            layout = new_layout;
            bean_offsets.push(offset);

            let type_id = definition.type_id;
            let name = definition.name;
            let id = next_id;

            type_id_bean_id_map.insert(type_id, id);
            name_bean_ids_map
                .entry(name)
                .or_insert_with(Vec::new)
                .push(id);

            next_id += 1;
        }

        let ptr = vec![0u8; layout.size()].into_boxed_slice();

        let bean_offsets = bean_offsets.into_boxed_slice();
        let bean_definitions = bean_definitions.into_boxed_slice();

        BeanContainer {
            ptr,
            bean_offsets,
            bean_definitions,
            type_id_bean_id_map,
            name_bean_ids_map,
            drop_methods,
            layout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct A;

    struct B;

    #[test]
    fn it_works() {
        
    }
}
