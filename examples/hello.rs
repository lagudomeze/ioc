// examples/hello.rs

use ioc::{Bean, export, InitCtx, run};
use ioc_core::Construct;

#[derive(Bean)]
#[bean(ioc_crate = ioc)]
struct B {
    #[inject(bean)]
    _a: &'static A,
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
#[bean(name = "aaa", ioc_crate = ioc)]
struct A {
    #[inject(config = "aaa.v")]
    _v: bool,
    #[inject(default)]
    _s: S,
}

#[derive(Bean)]
#[bean(construct = AnotherBeanA, ioc_crate = ioc)]
struct AnotherBeanA;

impl Construct for AnotherBeanA {
    type Bean = A;

    fn build(ctx: &mut InitCtx) -> ioc::Result<Self::Bean> {
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
