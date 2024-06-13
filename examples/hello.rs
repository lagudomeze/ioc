// examples/hello.rs

use ioc::{run, Bean};

#[derive(Bean)]
#[name("aaa")]
struct A;

#[derive(Bean)]
struct B {
    #[bean(crate::A)]
    _a: &'static A,
}
fn main() -> anyhow::Result<()> {
    run!();
    println!("{:p}", A::get());
    println!("{:p}", B::get());
    println!("{:p}", B::get()._a);
    Ok(())
}
