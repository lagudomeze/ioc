// examples/main

use ioc::{Bean, export, bean, BeanSpec, InitContext};

mod test;

mod tt {
    use ioc::Bean;

    #[derive(Bean)]
    pub struct Bxx {}
}

#[derive(Bean)]
pub struct B {
    #[inject(bean)]
    pub _a: &'static A,
    #[inject(bean_with = crate::A)]
    _a0: &'static A,
    #[inject(bean)]
    _a1: &'static A,
    #[inject(bean_with = AnotherBeanA)]
    _a2: &'static A,
}

#[allow(dead_code)]
struct S(&'static str);

impl Default for S {
    fn default() -> Self {
        Self("haha")
    }
}

#[derive(Bean)]
#[bean(name = "aaa")]
pub struct A {
    #[inject(config = "aaa.v")]
    _v: bool,
    #[inject(default)]
    _s: S,
}

struct AnotherBeanA;

#[bean]
impl BeanSpec for AnotherBeanA {
    type Bean = A;

    fn build<I>(ctx: &mut I) -> ioc::Result<Self::Bean>
    where
        I: InitContext,
    {
        Ok(A {
            _v: ctx.get_config::<_>("aaa.t")?,
            _s: S("hihi"),
        })
    }
}

export!();