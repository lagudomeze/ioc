// examples/hello.rs

use ioc::{run, Bean};

#[derive(Bean)]
#[name("aaa")]
struct A;

#[derive(Bean)]
struct B {
    #[bean(crate::A)]
    _a: &'static A,
    #[bean(crate::C)]
    _c: &'static C,
}


#[derive(Bean)]
struct C {
    #[bean(crate::A)]
    _a: &'static A,
    #[bean(crate::B)]
    _b: &'static B,
}

fn main() -> anyhow::Result<()> {
    run!();
    Ok(())
}
