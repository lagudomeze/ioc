use log::debug;

use crate::{
    Bean,
    BeanSpec,
    InitCtx,
    types::{BeanFamily, Method},
};

pub struct Init<'a>(std::marker::PhantomData<&'a ()>);

impl<'a> BeanFamily for Init<'a> {
    type Ctx = &'a mut InitCtx;

    type Method<B> = Wrapper<B, Self::Ctx>
    where
        B: Bean<Spec: 'static>;
}

pub struct Wrapper<T, C>(T, std::marker::PhantomData<C>);

impl<'a, B> Method<&'a mut InitCtx> for Wrapper<B, &'a mut InitCtx>
where
    B: Bean<Spec: 'static>,
{
    fn run(ctx: &'a mut InitCtx) -> crate::Result<&'a mut InitCtx> {
        ctx.get_or_init::<B>()?;
        debug!("Init bean of {} with type {}", B::Spec::name(), B::Spec::bean_type_name());
        Ok(ctx)
    }
}