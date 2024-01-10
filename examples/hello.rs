// examples/hello.rs

use ioc::{run, Bean, Ref};

#[derive(Bean)]
#[name("aaa")]
struct A;

#[derive(Bean)]
struct B {
    #[bean_ref()]
    _a: Ref<A>,
}
fn main() {
    run!();
}
