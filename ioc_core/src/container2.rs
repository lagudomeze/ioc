use std::{
    alloc::Layout,
    any::TypeId,
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::{Deref, Range},
    ptr::from_exposed_addr,
};

use thiserror::Error;

use crate::bean::{Bean, BeanDefinition};

#[derive(Error, Debug)]
pub enum ContainerError {
    #[error("No bean of type: {type_name}")]
    NoBean { type_name: &'static str },
    #[error(
        "Requred bean of type: {type_name} name: {target_name}, but candinates : {candinates:?}"
    )]
    NoNamedBean {
        type_name: &'static str,
        target_name: &'static str,
        candinates: Box<[&'static str]>,
    },
    #[error("Too many candinated bean! Type [{type_name}] of name {candinates:?}")]
    TooManyCandinatedBean {
        type_name: &'static str,
        candinates: Box<[&'static str]>,
    },
    #[error("Register duplicated bean! Type [{type_name}] name [{name}]")]
    DuplicateBeanDefiniton {
        type_name: &'static str,
        name: &'static str,
    },
    #[error(
        "Register duplicated bean! Name [{name}] of type [{type_name}, {duplicate_type_name}]"
    )]
    DuplicateBeanName {
        name: &'static str,
        type_name: &'static str,
        duplicate_type_name: &'static str,
    },
    #[error("loop dependency")]
    LoopDependency,
    #[error("unknown error")]
    Unknown,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct BeanId {
    id: usize,
}

impl BeanId {
    fn new(value: usize) -> Self {
        Self { id: value }
    }
}

pub struct BeanInfo {
    id: BeanId,
    dependencies: Box<[BeanId]>,
    level: usize,
}

pub struct ContainerInfo {
    bean_definitions: Box<[BeanDefinition]>,
    bean_offsets: Box<[Range<usize>]>,
    data_layout: Layout,
    type_bean_id_map: HashMap<TypeId, HashMap<&'static str, BeanId>>,
    name_bean_id_map: HashMap<&'static str, BeanId>,
    bean_infos: Box<[BeanInfo]>,
}

impl ContainerInfo {
    fn new(bean_definitions: Vec<BeanDefinition>) -> Result<Self, ContainerError> {
        let mut type_bean_id_map = HashMap::with_capacity(bean_definitions.len());
        let mut name_bean_id_map = HashMap::with_capacity(bean_definitions.len());
        let mut bean_offsets = Vec::with_capacity(bean_definitions.len());
        let mut data_layout = Layout::from_size_align(0, 8).unwrap();

        // 扫描所有的bean定义，找到重复定义的bean并报错，如果没有话，则构建type_id 和 name相关的map
        for (idx, definition) in bean_definitions.iter().enumerate() {

            let (new_layout, offset) = data_layout.extend(definition.layout).expect("");
            bean_offsets.push(offset..offset + definition.layout.size());
            data_layout = new_layout;

            let id: BeanId = BeanId::new(idx);
            if let Some(_) = type_bean_id_map
                .entry(definition.type_id)
                .or_insert(HashMap::with_capacity(16))
                .insert(definition.name, id)
            {
                return Err(ContainerError::DuplicateBeanDefiniton {
                    name: definition.name,
                    type_name: definition.type_name,
                });
            }
            if let Some(duplicate_idx) = name_bean_id_map.insert(definition.name, id) {
                let duplicate_definition = &bean_definitions[duplicate_idx.id];
                return Err(ContainerError::DuplicateBeanName {
                    name: definition.name,
                    type_name: definition.type_name,
                    duplicate_type_name: duplicate_definition.name,
                });
            };
        }

        let bean_offsets = bean_offsets.into_boxed_slice();
        let mut bean_infos = Vec::with_capacity(bean_definitions.len());
        let mut scan_ids = Vec::with_capacity(bean_definitions.len());

        for (idx, definition) in bean_definitions.iter().enumerate() {
            let mut dependencies = Vec::new();
            for dependency in definition.dependencies.iter() {
                if let Some(m) = type_bean_id_map.get(&dependency.type_id) {
                    if let Some(target_name) = dependency.name {
                        if let Some(id) = m.get(target_name) {
                            dependencies.push(*id);
                        } else {
                            let mut candinates: Vec<&'static str> = vec![];
                            for name in m.keys() {
                                candinates.push(*name);
                            }
                            return Err(ContainerError::NoNamedBean {
                                target_name,
                                type_name: definition.type_name,
                                candinates: candinates.into_boxed_slice(),
                            });
                        }
                    } else {
                        if m.len() == 1 {
                            let id = m.values().last().unwrap();
                            dependencies.push(*id);
                        } else {
                            let mut candinates: Vec<&'static str> = vec![];
                            for name in m.keys() {
                                candinates.push(*name);
                            }
                            return Err(ContainerError::TooManyCandinatedBean {
                                type_name: definition.type_name,
                                candinates: candinates.into_boxed_slice(),
                            });
                        }
                    }
                } else {
                    return Err(ContainerError::NoBean {
                        type_name: definition.type_name,
                    });
                }
            }
            let dependencies = dependencies.into_boxed_slice();
            scan_ids.push(idx);

            bean_infos.push(BeanInfo {
                id: BeanId::new(idx),
                dependencies,
                level: usize::MAX,
            });
        }

        let mut last = scan_ids.len() - 1;
        let mut ready_ids = HashSet::with_capacity(scan_ids.len());
        let mut level = 0;
        while last >= 0 {
            let mut some_ready = false;
            for i in 0..=last {
                let info = &mut bean_infos[scan_ids[i]];
                let mut ready = true;
                for id in info.dependencies.iter() {
                    if !ready_ids.contains(id) {
                        ready = false;
                        break;
                    }
                }
                if ready {
                    some_ready = true;
                    info.level = level;
                    ready_ids.insert(info.id);
                    scan_ids.swap(i, last);
                    last -= 1;
                }
            }
            if !some_ready {
                //todo
                return Err(ContainerError::LoopDependency);
            }
            level += 1;
        }

        bean_infos.sort_by(|l, r| l.level.cmp(&r.level));

        let bean_infos = bean_infos.into_boxed_slice();
        let bean_definitions = bean_definitions.into_boxed_slice();

        Ok(Self {
            data_layout,
            bean_offsets,
            bean_infos,
            bean_definitions,
            type_bean_id_map,
            name_bean_id_map,
        })
    }
}

impl ContainerInfo {
    pub fn offset_between(&self, from: BeanId, to: BeanId) -> isize {
        let from_offset = self.bean_offsets[from.id].start;
        let to_offset = self.bean_offsets[to.id].start;
        to_offset.wrapping_sub(from_offset) as isize
    }
}

pub struct Ref<T> {
    parent_offset: isize,
    inner_offset: isize,
    self_ptr: *const Ref<T>,
    marker: std::marker::PhantomData<T>,
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

struct ContainerBuilder {
    bean_definitions: Box<[BeanDefinition]>,
    bean_offsets: Box<[Range<usize>]>,
    type_id_bean_id_map: HashMap<TypeId, BeanId>,
    data_layout: Layout,
    data: Box<[u8]>,
    next_init_bean_id: usize,
}

pub trait BeanFactory {
    type T: Bean;

    fn build(&mut self) -> Self::T;
}

impl ContainerBuilder {
    fn new(bean_definitions: Vec<BeanDefinition>) -> Self {
        let bean_definitions = bean_definitions.into_boxed_slice();

        let mut data_layout = Layout::from_size_align(0, 8).unwrap();
        let mut type_id_bean_id_map = HashMap::with_capacity(16);
        let mut name_bean_ids_map = HashMap::with_capacity(16);
        let mut bean_offsets = Vec::with_capacity(bean_definitions.len());

        let mut next_id = 0;

        for definition in bean_definitions.iter() {
            let (new_layout, offset) = data_layout.extend(definition.layout).expect("");

            bean_offsets.push(offset..offset + definition.layout.size());

            data_layout = new_layout;

            let type_id = definition.type_id;
            let name = definition.name;
            let id = BeanId::new(next_id);

            type_id_bean_id_map.insert(type_id, id);
            name_bean_ids_map
                .entry(name)
                .or_insert_with(Vec::new)
                .push(id);

            next_id += 1;
        }

        let data = vec![0u8; data_layout.size()].into_boxed_slice();
        let bean_offsets = bean_offsets.into_boxed_slice();

        Self {
            bean_definitions,
            bean_offsets,
            type_id_bean_id_map,
            data_layout,
            data,
            next_init_bean_id: 0,
        }
    }

    fn init(&mut self) {}
}

#[cfg(test)]
mod tests {
    use std::{
        marker::PhantomData,
        mem::{offset_of, MaybeUninit},
        ptr,
    };

    use super::*;
    struct A {
        a: usize,
    }

    struct B {
        b: Vec<u8>,
    }

    struct C {
        f1: u8,
        f2: u16,
        ra: Ref<A>,
        rb: Ref<B>,
    }

    impl C {
        fn tttt(&mut self) {}
    }

    #[test]
    fn it_works() {
        let mut uninit = MaybeUninit::<(A, B, C)>::uninit();

        let a_ptr = unsafe { ptr::addr_of_mut!((*uninit.as_mut_ptr()).0) };
        let b_ptr = unsafe { ptr::addr_of_mut!((*uninit.as_mut_ptr()).1) };
        unsafe { a_ptr.write(A { a: 1 }) };
        unsafe { b_ptr.write(B { b: vec![2u8; 128] }) };

        let c_ptr = unsafe { ptr::addr_of_mut!((*uninit.as_mut_ptr()).2) };

        let offset_c_a = unsafe { a_ptr.byte_offset_from(c_ptr) } - offset_of!(C, ra) as isize;
        let offset_c_b = unsafe { b_ptr.byte_offset_from(c_ptr) } - offset_of!(C, rb) as isize;

        // let ra = Ref {
        //     offset: offset_c_a,
        //     marker: PhantomData::<A>,
        // };
        // let rb = Ref {
        //     offset: offset_c_b,
        //     marker: PhantomData::<B>,
        // };

        // unsafe {
        //     c_ptr.write(C {
        //         f1: 3,
        //         f2: 4,
        //         ra,
        //         rb,
        //     })
        // };

        // let init = unsafe { uninit.assume_init() };
        // println!("{:p} {:p}", &init.0.a, &init.2.ra.a);
        // println!("{:p} {:p}", &init.1.b, &init.2.rb.b);

        // let init = Box::new(init);
        // println!("{:p} {:p}", &init.0.a, &init.2.ra.a);
        // println!("{:p} {:p}", &init.1.b, &init.2.rb.b);

        // let a = init.2.ra;
    }
}
