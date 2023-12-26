use std::{
    alloc::Layout,
    any::{type_name, TypeId},
    collections::{HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, Range},
    ptr::from_exposed_addr,
};

use thiserror::Error;

use crate::bean::{Bean, BeanDefinition};

#[derive(Error, Debug)]
pub enum ContainerError {
    #[error("No bean of type: {type_name}")]
    NoBean { type_name: &'static str },
    #[error("No bean of type: {type_name} name: {target_name}")]
    NoBeanWithName {
        type_name: &'static str,
        target_name: &'static str,
    },
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
    pub fn new(bean_definitions: Vec<BeanDefinition>) -> Result<Self, ContainerError> {
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
    pub(crate) fn offset_between(&self, from: BeanId, to: BeanId) -> isize {
        let from_offset = self.bean_offsets[from.id].start;
        let to_offset = self.bean_offsets[to.id].start;
        to_offset.wrapping_sub(from_offset) as isize
    }

    pub fn find<T>(&self) -> Result<BeanId, ContainerError>
    where
        T: 'static + Bean,
    {
        let type_id = TypeId::of::<T>();
        let type_name = type_name::<T>();
        if let Some(map) = self.type_bean_id_map.get(&type_id) {
            if map.len() > 1 {
                let mut candinates = vec![];
                for name in map.keys() {
                    candinates.push(*name);
                }
                let candinates = candinates.into_boxed_slice();
                Err(ContainerError::TooManyCandinatedBean {
                    type_name,
                    candinates,
                })
            } else {
                Ok(*map.values().last().expect("not here"))
            }
        } else {
            Err(ContainerError::NoBean { type_name })
        }
    }

    pub fn find_by_name<T>(&self, name: &'static str) -> Result<BeanId, ContainerError>
    where
        T: 'static + Bean,
    {
        if let Some(id) = self.name_bean_id_map.get(name) {
            if self.bean_definitions[id.id].type_id == TypeId::of::<T>() {
                Ok(*id)
            } else {
                Err(ContainerError::NoBeanWithName {
                    type_name: type_name::<T>(),
                    target_name: name,
                })
            }
        } else {
            Err(ContainerError::NoBeanWithName {
                type_name: type_name::<T>(),
                target_name: name,
            })
        }
    }
}

pub struct Ref<T> {
    parent_offset: isize,
    inner_offset: isize,
    self_ptr: *const Ref<T>,
    marker: std::marker::PhantomData<T>,
}

impl<T> Ref<T>
where
    T: 'static + Bean,
{
    unsafe fn init<S>(
        ptr: *mut Ref<T>,
        inner_offset: isize,
        info: &ContainerInfo,
    ) -> Result<(), ContainerError>
    where
        S: 'static + Bean,
    {
        let from = info.find::<T>()?;
        let to = info.find::<S>()?;
        let parent_offset = info.offset_between(from, to);
        ptr.write(Ref {
            parent_offset,
            inner_offset,
            self_ptr: ptr,
            marker: PhantomData,
        });
        Ok(())
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

struct BeanContainer {
    info: ContainerInfo,
    data: Box<[u8]>,
}

impl BeanContainer {
    fn new(info: ContainerInfo) -> Self {
        let data = Vec::with_capacity(info.data_layout.size()).into_boxed_slice();
        Self { info, data }
    }

    pub(crate) fn mut_may_uninit<T>(&mut self) -> Result<&mut MaybeUninit<T>, ContainerError>
    where
        T: 'static + Bean,
    {
        let bean_id = self.info.find::<T>()?;
        let offset = self.info.bean_offsets[bean_id.id].clone();
        let mut_ptr = &self.data[offset].as_mut_ptr().cast::<T>();
        let maybe_uninnit = unsafe { mut_ptr.as_uninit_mut().unwrap_unchecked() };
        Ok(maybe_uninnit)
    }
}
#[cfg(test)]
mod tests {
    use std::{
        marker::PhantomData,
        mem::{offset_of, MaybeUninit},
        ptr::{self, addr_of_mut},
    };

    use crate::bean::Dependency;

    use super::*;
    struct A {
        a: usize,
    }

    impl Bean for A {}

    struct B {
        b: Vec<u8>,
    }

    impl Bean for B {
        fn name() -> &'static str {
            "haha_b"
        }
    }

    struct C {
        f1: u8,
        f2: u16,
        ra: Ref<A>,
        rb: Ref<B>,
    }

    impl Bean for C {
        fn dependencies() -> Vec<Dependency> {
            vec![Dependency::of::<A>(), Dependency::with_name::<B>("haha_b")]
        }
    }

    impl C {
        fn tttt(&mut self) {}
    }

    #[test]
    fn it_works() {
        let bean_definitions = vec![A::definition(), B::definition(), C::definition()];

        let info = ContainerInfo::new(bean_definitions).expect("haha");

        let mut container = BeanContainer::new(info);

        let a = container.mut_may_uninit::<A>().expect("haha");

        a.write(A { a: 0 });

        let b = container.mut_may_uninit::<B>().expect("haha");

        b.write(B { b: vec![1, 2, 3] });

        let c = container.mut_may_uninit::<C>().expect("haha");

        let ptr = c.as_mut_ptr();

        unsafe {
            addr_of_mut!((*ptr).f1).write(4u8);
        }
        unsafe {
            addr_of_mut!((*ptr).f2).write(5u16);
        }

        unsafe {
            let ra = addr_of_mut!((*ptr).ra);
            let inner_offset = 0isize.wrapping_sub(offset_of!(C, ra) as isize);
            Ref::init::<C>(ra, inner_offset, &container.info).unwrap();
        }

        unsafe {
            let rb = addr_of_mut!((*ptr).rb);
            let inner_offset = 0isize.wrapping_sub(offset_of!(C, rb) as isize);
            Ref::init::<C>(rb, inner_offset, &container.info).unwrap();
        }

        // let init = unsafe { uninit.assume_init() };
        // println!("{:p} {:p}", &init.0.a, &init.2.ra.a);
        // println!("{:p} {:p}", &init.1.b, &init.2.rb.b);

        // let init = Box::new(init);
        // println!("{:p} {:p}", &init.0.a, &init.2.ra.a);
        // println!("{:p} {:p}", &init.1.b, &init.2.rb.b);

        // let a = init.2.ra;
    }
}
