use std::marker::PhantomData;

use log::debug;

use crate::{Bean, Context};
use crate::types::{BeanFamily, MethodType};

pub struct Wrapper<'a, T>(PhantomData<&'a ()>,T);

pub struct Init<'a>(PhantomData<&'a ()>);

impl<'a> BeanFamily for Init<'a> {
    type Ctx = &'a mut Context;
    type Method<B> = Wrapper<'a, B> where B: Bean;
}

impl<'a, B> MethodType for Wrapper<'a, B>
where
    B: Bean,
{
    type Ctx = &'a mut Context;

    fn run(ctx: Self::Ctx) -> crate::Result<Self::Ctx>
    where
    {
        B::holder().get_or_try_init(|| B::build(ctx))?;
        debug!("Init bean of {} with type {}", B::name(), B::bean_type_name());
        Ok(ctx)
    }
}