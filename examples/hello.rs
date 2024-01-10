// examples/hello.rs

use ioc::{run, Bean, Ref};

#[derive(Bean)]
#[name("aaa")]
struct A;

#[derive(Bean)]
struct B {
    #[bean_ref("bbb")]
    _a: Ref<A>,
}
fn main() -> anyhow::Result<()> {
    run!();
    Ok(())
}
