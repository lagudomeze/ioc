use ioc::{Bean, run};
use success::{A, B};

fn main() -> anyhow::Result<()> {
    let _ = run!(
        deps(),
        self_crate = success,
        name = "success",
        dir = ".",
        profile = "dev",
        debug = true);
    println!("{:p}", A::get());
    println!("{:p}", B::get());
    println!("{:p}", B::get()._a);
    Ok(())
}
