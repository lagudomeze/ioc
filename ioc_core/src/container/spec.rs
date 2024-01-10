use std::{
    alloc::Layout,
    any::TypeId,
    collections::{HashMap, HashSet},
};

use super::{BeanId, ContainerError};
use crate::bean::{Bean, BeanDefinition, BeanQuery};

type Result<T> = std::result::Result<T, ContainerError>;

#[derive(Debug, Clone)]
pub struct BeanSpec {
    pub id: BeanId,
    dependencies: Box<[BeanId]>,
    level: usize,
}

type TypeBeanIdMap = HashMap<TypeId, HashMap<&'static str, BeanId>>;
type NameBeanIdMap = HashMap<&'static str, BeanId>;

#[derive(Debug)]
pub struct ContainerSpec {
    pub data_layout: Layout,
    pub bean_specs: Box<[BeanSpec]>,
    bean_definitions: Box<[BeanDefinition]>,
    bean_offsets: Box<[usize]>,
    type_bean_id_map: TypeBeanIdMap,
    name_bean_id_map: NameBeanIdMap,
}

#[derive(Debug)]
pub struct ContainerSpecBuilder {
    next_id: usize,
    definitions: Vec<BeanDefinition>,
    bean_offsets: Vec<usize>,
    type_bean_id_map: TypeBeanIdMap,
    name_bean_id_map: NameBeanIdMap,
    data_layout: Layout,
}

fn find(type_map: &TypeBeanIdMap, query: &BeanQuery) -> Result<BeanId> {
    match query {
        BeanQuery::OnlyType {
            type_id, type_name, ..
        } => {
            if let Some(m) = type_map.get(type_id) {
                if m.len() == 1 {
                    let id = m.values().last().expect("not here");
                    Ok(*id)
                } else {
                    let mut candinates: Vec<&'static str> = vec![];
                    for name in m.keys() {
                        candinates.push(*name);
                    }
                    return Err(ContainerError::TooManyCandinatedBean {
                        type_name,
                        candinates: candinates.into_boxed_slice(),
                    });
                }
            } else {
                return Err(ContainerError::NoBean { type_name });
            }
        }
        BeanQuery::NameAndType {
            name,
            type_id,
            type_name,
        } => {
            if let Some(m) = type_map.get(type_id) {
                if m.contains_key(name) {
                    return Ok(m[name]);
                } else {
                    return Err(ContainerError::NoBeanWithName {
                        type_name,
                        target_name: name,
                    });
                }
            } else {
                return Err(ContainerError::NoBean { type_name });
            }
        }
    }
}

impl ContainerSpecBuilder {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            definitions: Vec::with_capacity(16),
            bean_offsets: Vec::with_capacity(16),
            type_bean_id_map: HashMap::new(),
            name_bean_id_map: HashMap::new(),
            data_layout: Layout::from_size_align(0, 8).expect("not here"),
        }
    }

    pub fn append<B>(&mut self) -> Result<BeanId>
    where
        B: Bean + 'static,
    {
        let bean_id = BeanId::new(self.next_id);
        let definition = B::definition();

        let (new_layout, offset) = self.data_layout.extend(definition.layout)?;

        // duplicate name of bean definition
        if self.name_bean_id_map.contains_key(definition.name) {
            let duplicate_type_name = definition.type_name;
            return Err(ContainerError::DuplicateBeanName {
                name: definition.name,
                type_name: definition.type_name,
                duplicate_type_name,
            });
        }

        let duplicated = self
            .type_bean_id_map
            .entry(definition.type_id)
            .or_insert(HashMap::with_capacity(16))
            .insert(definition.name, bean_id);

        // because name is unique
        assert!(duplicated.is_none());

        self.name_bean_id_map.insert(definition.name, bean_id);

        self.definitions.push(definition);
        self.next_id += 1;
        self.data_layout = new_layout;
        self.bean_offsets.push(offset);

        Ok(bean_id)
    }

    pub fn build(self) -> Result<ContainerSpec> {
        let bean_definitions = self.definitions.into_boxed_slice();
        let type_bean_id_map = self.type_bean_id_map;
        let name_bean_id_map = self.name_bean_id_map;
        let data_layout = self.data_layout;
        let bean_offsets = self.bean_offsets.into_boxed_slice();

        let mut bean_specs = Vec::with_capacity(bean_definitions.len());
        let mut scan_ids = Vec::with_capacity(bean_definitions.len());

        for (idx, definition) in bean_definitions.iter().enumerate() {
            let mut dependencies = Vec::new();
            for dependency in definition.dependencies.iter() {
                let id = find(&type_bean_id_map, dependency)?;
                dependencies.push(id);
            }
            let dependencies = dependencies.into_boxed_slice();
            scan_ids.push(idx);

            bean_specs.push(BeanSpec {
                id: BeanId::new(idx),
                dependencies,
                level: usize::MAX,
            });
        }

        let mut ready_ids = HashSet::with_capacity(scan_ids.len());
        let mut tail = scan_ids.len();
        let mut level = 0;
        while tail > 0 {
            let mut some_ready = false;
            let mut head = 0;
            while head < tail {
                let info = &mut bean_specs[scan_ids[head]];
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
                    tail -= 1;
                    scan_ids.swap(head, tail);
                } else {
                    head += 1;
                }
            }
            if !some_ready {
                //todo
                return Err(ContainerError::LoopDependency);
            }
            level += 1;
        }

        bean_specs.sort_by(|l, r| l.level.cmp(&r.level));

        let bean_specs = bean_specs.into_boxed_slice();

        Ok(ContainerSpec {
            data_layout,
            bean_offsets,
            bean_specs,
            bean_definitions,
            type_bean_id_map,
            name_bean_id_map,
        })
    }
}

impl ContainerSpec {
    pub fn builder() -> ContainerSpecBuilder {
        ContainerSpecBuilder::new()
    }

    pub fn offset(&self, id: &BeanId) -> usize {
        self.bean_offsets[id.id]
    }

    pub fn query(&self, query: &BeanQuery) -> Result<BeanId> {
        match query {
            BeanQuery::OnlyType { type_id, type_name } => {
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
            BeanQuery::NameAndType {
                name,
                type_id,
                type_name,
            } => {
                if let Some(id) = self.name_bean_id_map.get(name) {
                    if self.bean_definitions[id.id].type_id == *type_id {
                        Ok(*id)
                    } else {
                        Err(ContainerError::NoBeanWithName {
                            type_name,
                            target_name: name,
                        })
                    }
                } else {
                    Err(ContainerError::NoBeanWithName {
                        type_name,
                        target_name: name,
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;

    use super::*;
    #[derive(Debug)]
    struct A {
        _a: usize,
    }

    impl Bean for A {}

    #[derive(Debug)]
    struct B {
        _b: Vec<u8>,
    }

    impl Bean for B {
        fn name() -> &'static str {
            "haha_b"
        }
    }

    #[derive(Debug)]
    struct C {
        _f1: u8,
        _f2: u16,
        _ra: NonNull<A>,
        _rb: NonNull<B>,
    }

    impl Bean for C {
        fn dependencies() -> Vec<BeanQuery> {
            vec![BeanQuery::of::<A>(), BeanQuery::named::<B>("haha_b")]
        }
    }

    #[test]
    fn it_works() {
        let mut builder = ContainerSpec::builder();
        builder.append::<C>().expect("");
        builder.append::<A>().expect("");
        builder.append::<B>().expect("");

        let _info = builder.build().expect("haha");
    }
}
