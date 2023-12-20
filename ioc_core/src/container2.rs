use std::{
    alloc::Layout,
    any::TypeId,
    collections::{HashMap, HashSet},
    ops::{Deref, Range},
    ptr::from_exposed_addr,
};

use thiserror::Error;

use crate::bean::{Bean, BeanDefinition, BeanId};

struct ContainerBuilder {
    bean_definitions: Box<[BeanDefinition]>,
    bean_offsets: Box<[Range<usize>]>,
    type_id_bean_id_map: HashMap<TypeId, BeanId>,
    data_layout: Layout,
    data: Box<[u8]>,
    next_init_bean_id: usize,
}

#[derive(Error, Debug)]
pub enum ContainerError {
    #[error("No bean of type: {type_name}")]
    NotRegisteredBean { type_name: &'static str },
    #[error("Duplcated bean type: {type_name} name: {name}")]
    DuplicateBeanDefiniton {
        type_name: &'static str,
        name: &'static str,
    },
    #[error("Duplcated bean name: {name} of type [{type_name}, {duplicate_type_name}]")]
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

pub trait BeanFactory {
    type T: Bean;

    fn build(&mut self) -> Self::T;
}

fn sort_by_dep(
    bean_definitions: Vec<BeanDefinition>,
) -> Result<Vec<BeanDefinition>, ContainerError> {
    let mut result = vec![];

    let mut type_id_idx_map = HashMap::with_capacity(bean_definitions.len());

    let mut name_idx_map = HashMap::with_capacity(bean_definitions.len());

    // 扫描所有的bean定义，找到重复定义的bean并报错，如果没有话，则构建type_id 和 name相关的map
    for (idx, definition) in bean_definitions.iter().enumerate() {
        if let Some(_) = type_id_idx_map
            .entry(&definition.type_id)
            .or_insert(HashMap::with_capacity(16))
            .insert(definition.name, idx)
        {
            return Err(ContainerError::DuplicateBeanDefiniton {
                name: definition.name,
                type_name: definition.type_name,
            });
        }
        if let Some(duplicate_idx) = name_idx_map.insert(definition.name, idx) {
            let duplicate_definition = &bean_definitions[duplicate_idx];
            return Err(ContainerError::DuplicateBeanName {
                name: definition.name,
                type_name: definition.type_name,
                duplicate_type_name: duplicate_definition.name,
            });
        };
    }

    let mut queue = vec![];

    let mut recompute_queue = vec![];

    // 扫描所有并计算入度
    for (idx, definition) in bean_definitions.iter().enumerate() {
        // 将入度为0的元素加入结果中
        if definition.dependencies.is_empty() {
            queue.swap_remove(idx);
        } else {
            // 其他归入重计算的数组
            recompute_queue.push(idx);
        }
    }

    let mut indegree: Vec<HashSet<usize>> = Vec::with_capacity(bean_definitions.len());

    // 拓扑排序
    while let Some(idx) = queue.pop() {
        let definition = bean_definitions[idx];

        recompute_queue.sw
        let type_id = definition.type_id;
        // for dependency_type_id in definition.dependencies.iter() {
        //     if  {

        //     }
        //     indegree[dependency as usize] -= 1;
        //     if indegree[dependency as usize] == 0 {
        //         queue.push(dependency as usize);
        //     }
        // }
        result.push(definition);
    }

    Ok(result)
}

impl ContainerBuilder {
    fn new(bean_definitions: Vec<BeanDefinition>) -> Self {
        let bean_definitions = sort_by_dep(bean_definitions).unwrap().into_boxed_slice();

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
            let id = next_id.into();

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

struct Ref<T> {
    offset: isize,
    marker: std::marker::PhantomData<T>,
}

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = self as *const Self;
        let target_addr = ptr.addr().wrapping_add_signed(self.offset);

        unsafe {
            from_exposed_addr::<Self::Target>(target_addr)
                .as_ref()
                .unwrap()
        }
    }
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

        let ra = Ref {
            offset: offset_c_a,
            marker: PhantomData::<A>,
        };
        let rb = Ref {
            offset: offset_c_b,
            marker: PhantomData::<B>,
        };

        unsafe {
            c_ptr.write(C {
                f1: 3,
                f2: 4,
                ra,
                rb,
            })
        };

        let init = unsafe { uninit.assume_init() };
        println!("{:p} {:p}", &init.0.a, &init.2.ra.a);
        println!("{:p} {:p}", &init.1.b, &init.2.rb.b);

        let init = Box::new(init);
        println!("{:p} {:p}", &init.0.a, &init.2.ra.a);
        println!("{:p} {:p}", &init.1.b, &init.2.rb.b);

        let a = init.2.ra;
    }
}
