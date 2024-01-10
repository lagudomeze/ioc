// examples/hello.rs

use ioc::{Bean, run_app, BEAN_COLLECTOR, BeanRegistry};
use ioc_core::Ref;
use linkme::distributed_slice;

#[derive(Bean)]
#[name("aaa")]
struct A;

#[derive(Bean)]
struct B{
    #[bean_ref()]
    _a: Ref<A>
}

#[distributed_slice(BEAN_COLLECTOR)]
fn register_bean_a(ctx: &mut BeanRegistry) {
    ctx.register::<A>(module_path!());
}

#[distributed_slice(BEAN_COLLECTOR)]
fn register_bean_b(ctx: &mut BeanRegistry) {
    ctx.register::<B>(module_path!());
}

fn main() {
    run_app();
}