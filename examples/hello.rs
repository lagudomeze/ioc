// examples/hello.rs

use std::sync::OnceLock;

use ioc::{Bean, run};
use ioc_core::{BeanFactory, Context};

#[allow(dead_code)]
struct S(&'static str);

impl Default for S {
    fn default() -> Self {
        Self("haha")
    }
}

#[derive(Bean)]
#[name("aaa")]
struct A {
    #[value("aaa.v")]
    _v: bool,
    _s: S,
}

struct AnotherBeanA;

impl BeanFactory for AnotherBeanA {
    type Bean = A;

    fn build(ctx: &mut Context) -> ioc_core::Result<Self::Bean> {
        Ok(A {
            _v: ctx.get_config::<_>("aaa.t")?,
            _s: S("hihi"),
        })
    }
}

impl Bean for AnotherBeanA {
    type Type = A;
    type Factory = AnotherBeanA;

    fn holder<'a>() -> &'a OnceLock<Self::Type> {
        static HOLDER: OnceLock<A> = OnceLock::new();
        &HOLDER
    }
}

#[derive(Bean)]
struct B {
    #[bean]
    _a: &'static A,
    #[bean(crate::A)]
    _a0: &'static A,
    #[bean()]
    _a1: &'static A,
    #[bean(AnotherBeanA)]
    _a2: &'static A,
}
fn main() -> anyhow::Result<()> {
    run!(config = "./", profile = "dev");
    println!("{:p}", A::get());
    println!("{:p}", B::get());
    println!("{:p}", B::get()._a);
    Ok(())
}
