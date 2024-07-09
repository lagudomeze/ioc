// examples/hello.rs

use ioc::{run, Bean, export};

#[derive(Bean)]
#[bean(name = "aaa", ioc_crate = "ioc")]
struct A;

#[derive(Bean)]
#[bean(name = "bbb", ioc_crate = "ioc")]
struct B {
    #[inject(bean_with = crate::A)]
    _a: &'static A,
    #[inject(bean_with = crate::C)]
    _c: &'static C,
}


#[derive(Bean)]
#[bean(name = "ccc", ioc_crate = "ioc")]
struct C {
    #[inject(bean_with = crate::A)]
    _a: &'static A,
    #[inject(bean)]
    _b: &'static B,
}
export!(root = "examples/failed.rs");

fn main() -> anyhow::Result<()> {
    run!();
    Ok(())
}
