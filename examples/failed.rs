// examples/hello.rs

use ioc::{run, Bean};

#[derive(Bean)]
#[name("aaa")]
struct A;

#[derive(Bean)]
struct B {
    #[inject(crate::A)]
    _a: &'static A,
    #[inject(crate::C)]
    _c: &'static C,
}


#[derive(Bean)]
struct C {
    #[inject(crate::A)]
    _a: &'static A,
    #[inject(crate::B)]
    _b: &'static B,
}

fn main() -> anyhow::Result<()> {
    run!();
    Ok(())
}
