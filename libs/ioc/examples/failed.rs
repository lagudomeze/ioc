// examples/hello.rs

use ioc::{export, run, Bean};

#[derive(Bean)]
#[bean(name = "ccc", ioc_crate = "ioc")]
struct C {
    #[inject(bean = crate::A)]
    _a: &'static A,
    #[inject(bean)]
    _b: &'static B,
}

#[derive(Bean)]
#[bean(name = "aaa", ioc_crate = "ioc")]
struct A {
    #[inject(bean)]
    _b: &'static B,
}

#[derive(Bean)]
#[bean(name = "bbb", ioc_crate = "ioc")]
struct B {
    #[inject(bean = crate::A)]
    _a: &'static A,
    #[inject(bean = crate::C)]
    _c: &'static C,
}
export!();

fn main() -> anyhow::Result<()> {
    run!();
    Ok(())
}
