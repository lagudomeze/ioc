pub(crate) mod spec;

use std::{
    alloc::LayoutError,
    marker::PhantomData,
    mem::{needs_drop, MaybeUninit},
    pin::Pin,
    ptr::{drop_in_place, NonNull},
};

use spec::{ContainerSpec, ContainerSpecBuilder};

use crate::{bean::{BeanQuery, BeanTypeHolder}, Bean};

use thiserror::Error;

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
    #[error("Container Layout error")]
    LayoutError(#[from] LayoutError),
    #[error("loop dependency")]
    LoopDependency,
    #[error("unknown error")]
    Unknown,
}

type Result<T> = std::result::Result<T, ContainerError>;

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct BeanId {
    id: usize,
}

impl BeanId {
    pub fn new(value: usize) -> Self {
        Self { id: value }
    }
}

pub struct Ref<T> {
    ptr: NonNull<T>,
    marker: PhantomData<T>,
}

impl<T> AsRef<T> for Ref<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<B> BeanTypeHolder for Ref<B> where B: Bean + 'static{
    type T = B;
}

pub trait BeanRetriever {
    fn retrieve(&self, qurey: &BeanQuery) -> Result<BeanId>;

    fn retrieve_ptr<T>(&self, id: &BeanId) -> Result<NonNull<T>>;

    fn make_ref<T: 'static>(&self, name: Option<&'static str>) -> Result<Ref<T>> {
        let id = self.retrieve(&BeanQuery::maybe_none_name::<T>(name))?;
        let ptr = self.retrieve_ptr(&id)?;
        Ok(Ref {
            ptr,
            marker: PhantomData::<T>,
        })
    }

    fn make_value<T>(&self, _path: &'static str) -> Result<T> {
        unimplemented!()
    }
}

pub trait BeanFactory {
    type T: Bean + Sized + 'static;

    unsafe fn unsafe_drop(ptr: *mut u8) {
        if needs_drop::<Self::T>() {
            drop_in_place(ptr.cast::<Self::T>());
        }
    }

    unsafe fn init<C>(ctx: &mut C) -> Result<()>
    where
        C: BeanRetriever,
    {
        let query = Self::T::self_qurey();
        let id = ctx.retrieve(&query)?;
        let ptr = ctx.retrieve_ptr::<Self::T>(&id)?;
        Self::init_in_place(ptr, ctx);
        Ok(())
    }

    unsafe fn init_in_place<C>(ptr: NonNull<Self::T>, ctx: &C)
    where
        C: BeanRetriever;
}

pub struct BeanContainer {
    spec: ContainerSpec,
    drop_methods: Box<[DropMethod]>,
    data: Pin<Box<[u8]>>,
}

impl Drop for BeanContainer {
    fn drop(&mut self) {
        for bean_spec in self.spec.bean_specs.iter().rev() {
            let offset = self.spec.offset(&bean_spec.id);
            let ptr = self.data[offset..].as_mut_ptr();
            unsafe {
                self.drop_methods[bean_spec.id.id](ptr);
            }
        }
    }
}

pub(crate) struct MaybeUninitBeanRetriever {
    spec: ContainerSpec,
    data: Box<[MaybeUninit<u8>]>,
}

impl MaybeUninitBeanRetriever {
    fn new(spec: ContainerSpec) -> Self {
        let size = spec.data_layout.size();
        Self {
            spec,
            data: Box::<[u8]>::new_uninit_slice(size),
        }
    }
}

impl BeanRetriever for MaybeUninitBeanRetriever {
    fn retrieve(&self, qurey: &BeanQuery) -> Result<BeanId> {
        self.spec.query(qurey)
    }

    fn retrieve_ptr<T>(&self, id: &BeanId) -> Result<NonNull<T>> {
        let offset = self.spec.offset(id);
        let ptr = self.data[offset..].as_ptr().cast::<T>();
        Ok(NonNull::new(ptr.cast_mut()).expect("Null ptr"))
    }
}

type DropMethod = unsafe fn(*mut u8);

type InitMethod = unsafe fn(&mut MaybeUninitBeanRetriever) -> Result<()>;

#[derive(Debug)]
pub struct BeanContainerBuilder {
    spec: ContainerSpecBuilder,
    drop_methods: Vec<DropMethod>,
    init_methods: Vec<InitMethod>,
}

impl BeanContainer {
    pub fn builder() -> BeanContainerBuilder {
        BeanContainerBuilder {
            spec: ContainerSpec::builder(),
            drop_methods: Vec::default(),
            init_methods: Vec::default(),
        }
    }
}

impl BeanContainerBuilder {
    pub fn append<'a, 'b, F: BeanFactory>(&mut self) -> Result<()> {
        self.spec.append::<F::T>()?;
        self.drop_methods.push(F::unsafe_drop);
        self.init_methods.push(F::init::<MaybeUninitBeanRetriever>);
        Ok(())
    }
}

impl BeanContainerBuilder {
    pub fn build(self) -> Result<BeanContainer> {
        let spec = self.spec.build()?;

        let bean_specs = spec.bean_specs.clone();

        let init_methods = self.init_methods.into_boxed_slice();

        let mut retriver = MaybeUninitBeanRetriever::new(spec);

        for bean_spec in bean_specs.iter() {
            let id = bean_spec.id;
            unsafe {
                let init_method = init_methods[id.id];
                init_method(&mut retriver)?;
            }
        }
        let data = Box::into_pin(unsafe { retriver.data.assume_init() });
        let drop_methods = self.drop_methods.into_boxed_slice();

        Ok(BeanContainer {
            spec: retriver.spec,
            data,
            drop_methods,
        })
    }
}
