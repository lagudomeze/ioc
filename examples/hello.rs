// examples/hello.rs

use ioc::{Bean, bean, BeanSpec, export, InitContext, run};

#[derive(Bean)]
#[bean(ioc_crate = ioc)]
struct B {
    #[inject(bean)]
    _a: &'static A,
    #[inject(bean= crate::A)]
    _a0: &'static A,
    #[inject(bean)]
    _a1: &'static A,
    #[inject(bean= AnotherBeanA)]
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
#[bean(name = "aaa", ioc_crate = ioc)]
struct A {
    #[inject(config = "aaa.v")]
    _v: bool,
    _s: S,
}

struct AnotherBeanA;

#[bean(name = "xxxx")]
impl BeanSpec for AnotherBeanA {
    type Bean = A;

    fn build(ctx: &mut impl InitContext) -> ioc::Result<Self::Bean> {
        Ok(A {
            _v: ctx.get_config::<_>("aaa.t")?,
            _s: S("hihi"),
        })
    }
}

export!(root = "examples/hello.rs");

fn main() -> anyhow::Result<()> {
    let _ = run!(
        debug = true;
        profile = "dev";
    );
    println!("{:p}", A::get());
    println!("{:p}", B::get());
    println!("{:p}", B::get()._a);
    Ok(())
}
