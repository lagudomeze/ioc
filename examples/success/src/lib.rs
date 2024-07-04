// examples/main

use ioc::{Bean, BeanFactory, Context, export};

mod test;

mod tt {
    use ioc::Bean;

    #[derive(Bean)]
    pub struct Bxx {}
}

#[derive(Bean)]
pub struct B {
    #[inject]
    pub _a: &'static A,
    #[inject(crate::A)]
    _a0: &'static A,
    #[inject()]
    _a1: &'static A,
    #[inject(AnotherBeanA)]
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
#[name("aaa")]
pub struct A {
    #[value("aaa.v")]
    _v: bool,
    _s: S,
}

#[derive(Bean)]
#[custom_factory]
struct AnotherBeanA;

impl BeanFactory for AnotherBeanA {
    type Bean = A;

    fn build(ctx: &mut Context) -> ioc::Result<Self::Bean> {
        Ok(A {
            _v: ctx.get_config::<_>("aaa.t")?,
            _s: S("hihi"),
        })
    }
}

export!(deps());