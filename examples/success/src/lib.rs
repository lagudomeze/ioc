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
    #[inject(bean = crate::A)]
    _a0: &'static A,
    #[inject(bean)]
    _a1: &'static A,
    #[inject(bean = AnotherBeanA)]
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
    #[inject(config(name = "aaa.v", default = true))]
    _v: bool,
    _s: S,
}

struct AnotherBeanA;

#[bean]
impl BeanSpec for AnotherBeanA {
    type Bean = A;

    fn build(ctx: &mut impl InitContext) -> ioc::Result<Self::Bean>
    {
        Ok(A {
            _v: ctx.get_config::<_>("aaa.t")?,
            _s: S("hihi"),
        })
    }
}

export!();