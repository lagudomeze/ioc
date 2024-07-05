use ioc::{Bean, run};
use success::{A, B};

fn main() -> anyhow::Result<()> {
    let _ = run!(crates(success), profile = "dev", debug = true, use_crate = false);
    println!("{:p}", A::get());
    println!("{:p}", B::get());
    println!("{:p}", B::get()._a);
    Ok(())
}
